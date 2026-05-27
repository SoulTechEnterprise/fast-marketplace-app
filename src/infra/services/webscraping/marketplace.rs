use async_trait::async_trait;
use chromiumoxide::cdp::browser_protocol::dom::SetFileInputFilesParams;
use chromiumoxide::{
    Element, Page,
    browser::{Browser, BrowserConfig},
};
use futures::StreamExt;
use std::path::PathBuf;
use tokio::time::{Duration, sleep};

use crate::domain::entities::models::property::category::Category as PropertyCategory;
use crate::domain::entities::models::property::model::Model as PropertyModel;

use crate::domain::entities::models::vehicle::bodystyle::BodyStyle as VehicleBodyStyle;
use crate::domain::entities::models::vehicle::category::Category as VehicleCategory;
use crate::domain::entities::models::vehicle::condition::Condition as VehicleCondition;
use crate::domain::entities::models::vehicle::fuel::Fuel as VehicleFuel;
use crate::domain::entities::models::vehicle::manufacturer::Manufacturer as VehicleManufacturer;
use crate::domain::{
    entities::{property::Property, vehicle::Vehicle},
    services::{error::DomainError, webscraping::marketplace::WebscrapingMarketplaceService},
};

const SEL_PHOTO_INPUT: &str = "input[type='file']";
const SEL_FACEBOOK_LOGGED_IN: &str = "div[aria-label='Facebook']";

const SEL_FACEBOOK_TRUST_DEVICE: &str = "div[data-testid='save-device-button'], \
                                          button[name='save_device'], \
                                          div[aria-label='Salvar dispositivo'], \
                                          .__7n5 button";

fn profile_dir(client_id: &str) -> PathBuf {
    #[cfg(target_os = "windows")]
    let base = dirs::data_local_dir().unwrap_or_else(|| PathBuf::from("."));
    #[cfg(not(target_os = "windows"))]
    let base = dirs::data_dir().unwrap_or_else(|| PathBuf::from("."));

    let dir = base
        .join("marketplace")
        .join("chrome-profiles")
        .join(client_id);
    std::fs::create_dir_all(&dir).ok();
    dir
}

#[async_trait]
pub trait PageExt {
    async fn wait_for_element(&self, selector: &str) -> Result<Element, DomainError>;
    async fn wait_for_xpath(&self, xpath: &str) -> Result<Element, DomainError>;
    async fn click_option(&self, text: &str) -> Result<(), DomainError>;
    async fn focus_and_type(&self, xpath: &str, value: &str) -> Result<(), DomainError>;
    async fn select_dropdown(&self, xpath: &str, option_text: &str) -> Result<(), DomainError>;
}

#[async_trait]
impl PageExt for Page {
    async fn wait_for_element(&self, selector: &str) -> Result<Element, DomainError> {
        for _ in 1..=20 {
            if let Ok(el) = self.find_element(selector).await {
                return Ok(el);
            }
            sleep(Duration::from_millis(500)).await;
        }
        Err(DomainError::AutomationError(format!(
            "Elemento não carregou na tela: {}",
            selector
        )))
    }

    async fn wait_for_xpath(&self, xpath: &str) -> Result<Element, DomainError> {
        for _ in 1..=20 {
            if let Ok(el) = self.find_xpath(xpath).await {
                return Ok(el);
            }
            sleep(Duration::from_millis(500)).await;
        }
        Err(DomainError::AutomationError(format!(
            "XPath não carregou na tela: {}",
            xpath
        )))
    }

    async fn click_option(&self, text: &str) -> Result<(), DomainError> {
        let xpath = format!("//*[@role='option'][contains(., '{}')]", text);
        let el = self.wait_for_xpath(&xpath).await?;
        el.click().await.map_err(|e| {
            DomainError::AutomationError(format!("Falha ao clicar na opção '{}': {}", text, e))
        })?;
        sleep(Duration::from_millis(300)).await;
        Ok(())
    }

    async fn focus_and_type(&self, xpath: &str, value: &str) -> Result<(), DomainError> {
        let el = self.wait_for_xpath(xpath).await?;
        el.click().await.map_err(|e| {
            DomainError::AutomationError(format!(
                "Falha ao clicar no input para focar ({}): {}",
                xpath, e
            ))
        })?;

        let js = format!(
            r#"(function() {{
                var el = document.evaluate({:?}, document, null,
                    XPathResult.FIRST_ORDERED_NODE_TYPE, null).singleNodeValue;
                if (!el) return false;
                el.focus();
                document.execCommand('insertText', false, {:?});
                return true;
            }})()"#,
            xpath, value
        );

        let ok = self
            .evaluate(js)
            .await
            .map_err(|e| {
                DomainError::AutomationError(format!(
                    "Falha ao injetar JS no input ({}): {}",
                    xpath, e
                ))
            })?
            .into_value::<bool>()
            .unwrap_or(false);

        if !ok {
            return Err(DomainError::AutomationError(format!(
                "O JS de digitação falhou no elemento: {}",
                xpath
            )));
        }

        Ok(())
    }

    async fn select_dropdown(&self, xpath: &str, option_text: &str) -> Result<(), DomainError> {
        let el = self.wait_for_xpath(xpath).await?;
        el.click().await.map_err(|e| {
            DomainError::AutomationError(format!("Falha ao clicar no dropdown ({}): {}", xpath, e))
        })?;
        sleep(Duration::from_secs(1)).await;
        self.click_option(option_text).await?;
        Ok(())
    }
}

struct BrowserGuard {
    browser: Option<Browser>,
}

impl BrowserGuard {
    fn new(browser: Browser) -> Self {
        Self { browser: Some(browser) }
    }

    async fn close(mut self) {
        if let Some(mut browser) = self.browser.take() {
            let _ = browser.kill().await;
        }
    }
}

impl Drop for BrowserGuard {
    fn drop(&mut self) {
        if let Some(mut browser) = self.browser.take() {
            tokio::task::spawn(async move {
                let _ = browser.kill().await;
            });
        }
    }
}

pub struct FacebookMarketplaceService {}

impl FacebookMarketplaceService {
    pub fn new() -> Self {
        Self {}
    }

    async fn launch_browser(client_id: &str) -> Result<Browser, DomainError> {
        let (browser, mut handler) = Browser::launch(
            BrowserConfig::builder()
                .with_head()
                .user_data_dir(profile_dir(client_id))
                .arg("--start-maximized")
                .arg("--window-size=1280,720")
                .viewport(chromiumoxide::handler::viewport::Viewport {
                    width: 1280,
                    height: 720,
                    device_scale_factor: Some(1.0),
                    emulating_mobile: false,
                    is_landscape: true,
                    has_touch: false,
                })
                .arg("--disable-infobars")
                .arg("--disable-notifications")
                .arg("--disable-blink-features=AutomationControlled")
                .arg("--no-sandbox")
                .arg("--disable-dev-shm-usage")
                .arg("--disable-web-security")
                .arg("--disable-features=IsolateOrigins,site-per-process")
                .arg("--allow-running-insecure-content")
                .arg("--disable-site-isolation-trials")
                .arg("--excludeSwitches=enable-automation")
                .arg("--useAutomationExtension=false")
                .arg("--user-agent=Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/136.0.0.0 Safari/537.36")
                .arg("--no-restore-session-state")
                .arg("--restore-last-session=false")
                .arg("--disable-session-crashed-bubble")
                .arg("--disable-background-mode")
                .arg("--disable-automation")          // remove a barra "being controlled"
                .arg("--password-store=basic")
                .arg("--use-mock-keychain")
                .arg("--lang=pt-BR")
                .arg("--disable-ipc-flooding-protection")
                .arg("--disable-renderer-backgrounding")
                .arg("--disable-backgrounding-occluded-windows")
                .arg("--disable-client-side-phishing-detection")
                .arg("--disable-crash-reporter")
                .arg("--disable-oopr-debug-crash-dump")
                .arg("--no-crash-upload")
                .arg("--hide-crash-restore-bubble")   // remove o "Restore pages?"
                .arg("--suppress-message-center-popups")
                .arg("--disable-popup-blocking")
                .arg("--no-first-run")
                .arg("--no-default-browser-check")
                .arg("--new-window")
                .build()
                .map_err(|e| DomainError::AutomationError(format!("Falha ao construir configuração do browser: {}", e)))?,
        )
        .await
        .map_err(|e| {
            eprintln!("Erro ao iniciar Google Chrome: {:?}", e);
            DomainError::AutomationError("Google Chrome não foi encontrado no sistema ou falhou ao iniciar. Certifique-se de que o Google Chrome original está instalado no computador.".to_string())
        })?;

        tokio::task::spawn(async move {
            while let Some(h) = handler.next().await {
                if h.is_err() {
                    break;
                }
            }
        });

        Ok(browser)
    }

    async fn launch_browser_headless(client_id: &str) -> Result<Browser, DomainError> {
        let (browser, mut handler) = Browser::launch(
            BrowserConfig::builder()
                .user_data_dir(profile_dir(client_id))
                .build()
                .map_err(|e| DomainError::AutomationError(format!("Falha ao construir configuração headless do browser: {}", e)))?,
        )
        .await
        .map_err(|e| {
            eprintln!("Erro ao iniciar Google Chrome Headless: {:?}", e);
            DomainError::AutomationError("Google Chrome não foi encontrado no sistema ou falhou ao iniciar em modo oculto.".to_string())
        })?;

        tokio::task::spawn(async move {
            while let Some(h) = handler.next().await {
                if h.is_err() {
                    break;
                }
            }
        });

        Ok(browser)
    }

    async fn get_or_create_page(browser: &Browser, url: &str) -> Result<Page, DomainError> {
        let mut page = None;
        for _ in 0..15 {
            if let Ok(pages) = browser.pages().await {
                if !pages.is_empty() {
                    page = Some(pages.into_iter().next().unwrap());
                    break;
                }
            }
            sleep(Duration::from_millis(100)).await;
        }

        let page = match page {
            Some(p) => {
                p.goto(url).await.map_err(|e| DomainError::AutomationError(format!("Falha ao navegar para a página: {}", e)))?;
                p
            }
            None => {
                browser
                    .new_page(url)
                    .await
                    .map_err(|e| DomainError::AutomationError(format!("Falha ao criar nova página: {}", e)))?
            }
        };

        Ok(page)
    }
}

impl Default for FacebookMarketplaceService {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl WebscrapingMarketplaceService for FacebookMarketplaceService {
    async fn add_property(&self, entity: Property, client_id: String) -> Result<(), DomainError> {
        const XPATH_MODEL_DROPDOWN: &str =
            "//label[@role='combobox'][contains(., 'venda ou locação')]";
        const XPATH_CATEGORY_DROPDOWN: &str =
            "//label[@role='combobox'][contains(., 'Tipo de imóvel')]";
        const XPATH_PARKING_DROPDOWN: &str =
            "//label[@role='combobox'][contains(., 'Vagas de estacionamento')]";
        const XPATH_BEDROOM_INPUT: &str =
            "//span[contains(., 'Número de quartos')]/following::input[1]";
        const XPATH_BATHROOM_INPUT: &str =
            "//span[contains(., 'Número de banheiros')]/following::input[1]";
        const XPATH_PRICE_INPUT: &str = "//span[contains(., 'Preço')]/following::input[1]";
        const XPATH_ADDRESS_INPUT: &str = "//input[@role='combobox'][@aria-autocomplete='list'][not(contains(@aria-label, 'Pesquisar'))]";
        const XPATH_DESCRIPTION_TEXTAREA: &str =
            "//span[contains(., 'Descrição do imóvel')]/following::textarea[1]";
        const XPATH_METER_INPUT: &str =
            "//span[contains(., 'Metros quadrados')]/following::input[1]";
        const XPATH_TAX_INPUT: &str = "//span[contains(., 'Imposto')]/following::input[1]";
        const XPATH_CONDOMINIUM_INPUT: &str =
            "//span[contains(., 'Condomínio')]/following::input[1]";

        let url = "https://www.facebook.com/marketplace/create/rental?locale=pt_BR".to_string();

        let browser = Self::launch_browser(client_id.as_str()).await?;
        let guard = BrowserGuard::new(browser);
        let page = Self::get_or_create_page(guard.browser.as_ref().unwrap(), &url).await?;

        page.evaluate(
            r#"
                Object.defineProperty(navigator, 'webdriver', {
                    get: () => undefined,
                    configurable: true
                });

                delete window.cdc_adoQpoasnfa76pfcZLmcfl_Array;
                delete window.cdc_adoQpoasnfa76pfcZLmcfl_Promise;
                delete window.cdc_adoQpoasnfa76pfcZLmcfl_Symbol;

                Object.defineProperty(navigator, 'plugins', {
                    get: () => [1, 2, 3, 4, 5],
                });

                Object.defineProperty(navigator, 'languages', {
                    get: () => ['pt-BR', 'pt', 'en-US', 'en'],
                });
            "#,
        )
        .await
        .map_err(|e| {
            DomainError::AutomationError(format!(
                "Falha ao simular uma pessoa real para o Chromium: {}",
                e
            ))
        })?;

        let el = page.wait_for_element(SEL_PHOTO_INPUT).await?;
        let image_paths: Vec<String> = entity.image().iter().map(|s| s.to_string()).collect();

        page.execute(SetFileInputFilesParams {
            files: image_paths,
            node_id: Some(el.node_id),
            backend_node_id: None,
            object_id: None,
        })
        .await
        .map_err(|e| {
            DomainError::AutomationError(format!("Falha ao enviar as fotos para o Chromium: {}", e))
        })?;

        sleep(Duration::from_secs(2)).await;

        page.select_dropdown(
            XPATH_MODEL_DROPDOWN,
            PropertyModel::transform(entity.model()),
        )
        .await?;
        page.select_dropdown(
            XPATH_CATEGORY_DROPDOWN,
            PropertyCategory::transform(entity.category()),
        )
        .await?;

        page.focus_and_type(XPATH_BEDROOM_INPUT, &entity.bedroom().to_string())
            .await?;
        page.focus_and_type(XPATH_BATHROOM_INPUT, &entity.bathroom().to_string())
            .await?;
        page.focus_and_type(XPATH_PRICE_INPUT, &entity.price().to_string())
            .await?;

        page.focus_and_type(XPATH_ADDRESS_INPUT, entity.address())
            .await?;
        sleep(Duration::from_millis(800)).await;

        if let Ok(el) = page.find_xpath("//*[@role='option'][1]").await {
            let _ = el.click().await;
        }

        page.focus_and_type(XPATH_DESCRIPTION_TEXTAREA, entity.description())
            .await?;
        page.focus_and_type(XPATH_METER_INPUT, &entity.meter().to_string())
            .await?;
        page.focus_and_type(XPATH_TAX_INPUT, &entity.tax().to_string())
            .await?;
        page.focus_and_type(XPATH_CONDOMINIUM_INPUT, &entity.condominium().to_string())
            .await?;

        page.select_dropdown(XPATH_PARKING_DROPDOWN, &entity.parking().to_string())
            .await?;

        let max_attempts = 240;
        let mut success = false;
        for _ in 0..max_attempts {
            sleep(Duration::from_secs(2)).await;
            match page.evaluate("window.location.href").await {
                Err(_) => {
                    break;
                }
                Ok(js_result) => {
                    if let Ok(current_url) = js_result.into_value::<String>() {
                        if current_url.contains("marketplace/you/selling")
                            || current_url.contains("marketplace/you/vehicles")
                        {
                            success = true;
                            break;
                        }
                    }
                }
            }
        }

        guard.close().await;

        if !success {
            return Err(DomainError::AutomationError(
                "A publicação falhou, o tempo esgotou ou a janela foi fechada prematuramente.".to_string(),
            ));
        }

        Ok(())
    }

    async fn add_vehicle(&self, entity: Vehicle, client_id: String) -> Result<(), DomainError> {
        const XPATH_TYPE_DROPDOWN: &str =
            "//label[@role='combobox'][contains(., 'Tipo de veículo')]";
        const XPATH_YEAR_DROPDOWN: &str = "//label[@role='combobox'][contains(., 'Ano')]";
        const XPATH_MAKE_DROPDOWN: &str = "//label[@role='combobox'][contains(., 'Fabricante')]";
        const XPATH_MAKE_INPUT: &str = "//span[contains(., 'Fabricante')]/following::input[1]";
        const XPATH_MODEL_INPUT: &str = "//span[contains(., 'Modelo')]/following::input[1]";
        const XPATH_MILEAGE_INPUT: &str =
            "//span[contains(., 'Quilometragem')]/following::input[1]";
        const XPATH_PRICE_INPUT: &str = "//span[contains(., 'Preço')]/following::input[1]";
        const XPATH_BODYSTYLE_DROPDOWN: &str =
            "//label[@role='combobox'][contains(., 'Estilo da carroceria')]";
        const XPATH_CONDITION_DROPDOWN: &str =
            "//label[@role='combobox'][contains(., 'Condição do veículo')]";
        const XPATH_FUEL_DROPDOWN: &str =
            "//label[@role='combobox'][contains(., 'Tipo de combustível')]";
        const XPATH_LOCATION_INPUT: &str = "//input[@role='combobox'][@aria-label='Localização']";
        const XPATH_DESCRIPTION_TEXTAREA: &str =
            "//span[contains(., 'Descrição')]/following::textarea[1]";
        const SEL_PHOTO_INPUT: &str = "input[type='file'][accept*='image']";

        let url = "https://www.facebook.com/marketplace/create/vehicle?locale=pt_BR".to_string();

        let browser = Self::launch_browser(client_id.as_str()).await?;
        let guard = BrowserGuard::new(browser);
        let page = Self::get_or_create_page(guard.browser.as_ref().unwrap(), &url).await?;

        page.evaluate(
            r#"
                Object.defineProperty(navigator, 'webdriver', {
                    get: () => undefined,
                    configurable: true
                });

                delete window.cdc_adoQpoasnfa76pfcZLmcfl_Array;
                delete window.cdc_adoQpoasnfa76pfcZLmcfl_Promise;
                delete window.cdc_adoQpoasnfa76pfcZLmcfl_Symbol;

                Object.defineProperty(navigator, 'plugins', {
                    get: () => [1, 2, 3, 4, 5],
                });

                Object.defineProperty(navigator, 'languages', {
                    get: () => ['pt-BR', 'pt', 'en-US', 'en'],
                });
            "#,
        )
        .await
        .map_err(|e| {
            DomainError::AutomationError(format!(
                "Falha ao simular uma pessoa real para o Chromium: {}",
                e
            ))
        })?;

        let el = page.wait_for_element(SEL_PHOTO_INPUT).await?;
        let image_paths: Vec<String> = entity.image().iter().map(|s| s.to_string()).collect();

        page.execute(SetFileInputFilesParams {
            files: image_paths,
            node_id: Some(el.node_id),
            backend_node_id: None,
            object_id: None,
        })
        .await
        .map_err(|e| {
            DomainError::AutomationError(format!("Falha ao enviar as fotos para o Chromium: {}", e))
        })?;

        sleep(Duration::from_secs(2)).await;

        page.select_dropdown(
            XPATH_TYPE_DROPDOWN,
            VehicleCategory::transform(entity.category()),
        )
        .await?;

        sleep(Duration::from_secs(2)).await;

        page.select_dropdown(XPATH_YEAR_DROPDOWN, &entity.year().to_string())
            .await?;

        match entity.category() {
            VehicleCategory::CarOrPickup
            | VehicleCategory::Motorcycle
            | VehicleCategory::CommercialOrIndustrial => {
                page.select_dropdown(
                    XPATH_MAKE_DROPDOWN,
                    VehicleManufacturer::transform(entity.manufacturer()),
                )
                .await?;
            }
            _ => {
                page.focus_and_type(
                    XPATH_MAKE_INPUT,
                    VehicleManufacturer::transform(entity.manufacturer()),
                )
                .await?;
            }
        }

        page.focus_and_type(XPATH_MODEL_INPUT, &entity.model())
            .await?;

        if page.find_xpath(XPATH_MILEAGE_INPUT).await.is_ok() {
            let _ = page
                .focus_and_type(XPATH_MILEAGE_INPUT, &entity.mileage().to_string())
                .await;
        }

        if page.find_xpath(XPATH_BODYSTYLE_DROPDOWN).await.is_ok() {
            let _ = page
                .select_dropdown(
                    XPATH_BODYSTYLE_DROPDOWN,
                    VehicleBodyStyle::transform(entity.bodystyle()),
                )
                .await;
        }

        if page.find_xpath(XPATH_CONDITION_DROPDOWN).await.is_ok() {
            let _ = page
                .select_dropdown(
                    XPATH_CONDITION_DROPDOWN,
                    VehicleCondition::transform(entity.condition()),
                )
                .await;
        }

        if page.find_xpath(XPATH_FUEL_DROPDOWN).await.is_ok() {
            let _ = page
                .select_dropdown(XPATH_FUEL_DROPDOWN, VehicleFuel::transform(entity.fuel()))
                .await;
        }

        page.focus_and_type(XPATH_PRICE_INPUT, &entity.price().to_string())
            .await?;

        page.focus_and_type(XPATH_LOCATION_INPUT, &entity.address())
            .await?;
        sleep(Duration::from_millis(800)).await;

        if let Ok(el) = page.find_xpath("//*[@role='option'][1]").await {
            let _ = el.click().await;
        }

        if page.find_xpath(XPATH_DESCRIPTION_TEXTAREA).await.is_ok() {
            let _ = page
                .focus_and_type(XPATH_DESCRIPTION_TEXTAREA, &entity.description())
                .await;
        }

        let max_attempts = 240;
        let mut success = false;
        for _ in 0..max_attempts {
            sleep(Duration::from_secs(2)).await;
            match page.evaluate("window.location.href").await {
                Err(_) => {
                    break;
                }
                Ok(js_result) => {
                    if let Ok(current_url) = js_result.into_value::<String>() {
                        if current_url.contains("marketplace/you/selling")
                            || current_url.contains("marketplace/you/vehicles")
                        {
                            success = true;
                            break;
                        }
                    }
                }
            }
        }

        guard.close().await;

        if !success {
            return Err(DomainError::AutomationError(
                "A publicação falhou, o tempo esgotou ou a janela foi fechada prematuramente.".to_string(),
            ));
        }

        Ok(())
    }

    async fn signin(&self, client_id: String) -> Result<(), DomainError> {
        let browser = Self::launch_browser(client_id.as_str()).await?;
        let guard = BrowserGuard::new(browser);
        let page = Self::get_or_create_page(guard.browser.as_ref().unwrap(), "https://www.facebook.com/login?locale=pt_BR").await?;

        for _ in 0..240 {
            sleep(Duration::from_secs(2)).await;
            if let Ok(js_result) = page.evaluate("window.location.href").await {
                if let Ok(current_url) = js_result.into_value::<String>() {
                    let is_out_of_login = !current_url.contains("login")
                        && !current_url.contains("two_factor")
                        && !current_url.contains("two-factor")
                        && !current_url.contains("save-device")
                        && !current_url.contains("trust");

                    if is_out_of_login && page.find_element(SEL_FACEBOOK_LOGGED_IN).await.is_ok() {
                        let trust_prompt_visible =
                            page.find_element(SEL_FACEBOOK_TRUST_DEVICE).await.is_ok();

                        if trust_prompt_visible {
                            continue;
                        }

                        sleep(Duration::from_secs(8)).await;
                        guard.close().await;
                        sleep(Duration::from_secs(2)).await;
                        return Ok(());
                    }
                }
            }
        }

        Err(DomainError::NotFound)
    }

    async fn signout(&self, client_id: String) -> Result<(), DomainError> {
        let browser = Self::launch_browser_headless(client_id.as_str()).await?;
        let guard = BrowserGuard::new(browser);
        let page = Self::get_or_create_page(guard.browser.as_ref().unwrap(), "https://www.facebook.com/?locale=pt_BR").await?;

        sleep(Duration::from_secs(6)).await;

        if page.find_element(SEL_FACEBOOK_LOGGED_IN).await.is_err() {
            return Err(DomainError::NotFound);
        }

        // Extrai o token h do form de logout e submete
        let _ = page
            .evaluate(
                r#"
                (function() {
                    const form = document.querySelector('form[action*="logout.php"]');
                    if (!form) return { success: false, reason: 'form not found' };

                    const h = form.querySelector('input[name="h"]');
                    const ref_ = form.querySelector('input[name="ref"]');
                    if (!h) return { success: false, reason: 'token h not found' };

                    const params = new URLSearchParams();
                    params.append('h', h.value);
                    params.append('ref', ref_ ? ref_.value : 'mb');

                    fetch('/logout.php?button_location=settings&button_name=logout', {
                        method: 'POST',
                        credentials: 'include',
                        headers: { 'Content-Type': 'application/x-www-form-urlencoded' },
                        body: params.toString()
                    });

                    return { success: true, h: h.value };
                })()
            "#,
            )
            .await
            .map_err(|e| {
                DomainError::AutomationError(format!("Falha ao executar logout: {}", e))
            })?;

        for _ in 0..20 {
            sleep(Duration::from_secs(2)).await;
            if let Ok(js_result) = page.evaluate("window.location.href").await {
                if let Ok(current_url) = js_result.into_value::<String>() {
                    if current_url.contains("login")
                        || current_url.contains("logged_out")
                        || current_url.contains("checkpoint")
                        || current_url.contains("accounts/login")
                    {
                        sleep(Duration::from_secs(3)).await;
                        guard.close().await;
                        return Ok(());
                    }

                    // Tela de seleção de conta — força navegação para login limpo
                    if current_url.contains("facebook.com") && !current_url.contains("login") {
                        page.goto("https://www.facebook.com/login?next&prompt=select_account&login_attempt=1&lwv=100&locale=pt_BR")
                            .await
                            .ok();
                        sleep(Duration::from_secs(2)).await;
                        guard.close().await;
                        return Ok(());
                    }
                }
            }
        }

        Err(DomainError::AutomationError(
            "Timeout: logout não foi confirmado".to_string(),
        ))
    }

    async fn get_account(&self, client_id: String) -> Result<bool, DomainError> {
        let browser = Self::launch_browser_headless(client_id.as_str()).await?;
        let guard = BrowserGuard::new(browser);

        let page = Self::get_or_create_page(guard.browser.as_ref().unwrap(), "https://www.facebook.com/?locale=pt_BR").await?;

        sleep(Duration::from_secs(6)).await;

        let is_logged_in = page.find_element(SEL_FACEBOOK_LOGGED_IN).await.is_ok();

        guard.close().await;
        Ok(is_logged_in)
    }
}
