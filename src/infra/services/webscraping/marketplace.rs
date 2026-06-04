use async_trait::async_trait;
use chromiumoxide::cdp::browser_protocol::dom::SetFileInputFilesParams;
use chromiumoxide::{
    Element, Page,
    browser::{Browser, BrowserConfig},
};
use futures::StreamExt;
use std::path::PathBuf;
use tokio::time::{Duration, sleep};

use crate::infra::logger;

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

/// Validates client_id to prevent path traversal and injection attacks.
/// Only allows alphanumeric characters, hyphens, and underscores.
fn sanitize_client_id(client_id: &str) -> Result<&str, DomainError> {
    if client_id.is_empty()
        || !client_id
            .chars()
            .all(|c| c.is_alphanumeric() || c == '_' || c == '-')
    {
        return Err(DomainError::AutomationError(
            "Invalid client_id format".to_string(),
        ));
    }
    Ok(client_id)
}

const SEL_PHOTO_INPUT: &str = "input[type='file']";
const SEL_FACEBOOK_LOGGED_IN: &str = "div[aria-label='Facebook']";

const SEL_FACEBOOK_TRUST_DEVICE: &str = "div[data-testid='save-device-button'], \
                                          button[name='save_device'], \
                                          div[aria-label='Salvar dispositivo'], \
                                          .__7n5 button";

fn cleanup_stale_lock_files(dir: &std::path::Path) {
    // Only delete leftover lock files that prevent Chrome from starting.
    // We no longer forcefully kill Chrome processes here because the
    // graceful browser.close() + wait() in BrowserGuard handles that.
    // Forcefully killing Chrome was causing cookie/session data loss.
    let _ = std::fs::remove_file(dir.join("SingletonLock"));
    let _ = std::fs::remove_file(dir.join("SingletonSocket"));
    let _ = std::fs::remove_file(dir.join("SingletonCookie"));
}

fn profile_dir(client_id: &str) -> PathBuf {
    #[cfg(target_os = "windows")]
    let base = dirs::data_local_dir().unwrap_or_else(|| PathBuf::from("."));
    #[cfg(not(target_os = "windows"))]
    let base = dirs::data_dir().unwrap_or_else(|| PathBuf::from("."));

    let dir = base
        .join("marketplace")
        .join("chrome-profiles")
        .join(client_id);

    if let Err(e) = std::fs::create_dir_all(&dir) {
        logger::warn(&format!(
            "Falha ao criar pasta de perfil: {}. Usando diretório temporário.",
            e
        ));
        let temp_dir = std::env::temp_dir()
            .join("marketplace-chrome-profiles")
            .join(client_id);
        let _ = std::fs::create_dir_all(&temp_dir);
        cleanup_stale_lock_files(&temp_dir);
        return temp_dir;
    }

    cleanup_stale_lock_files(&dir);

    dir
}

fn get_option_xpath(text: &str) -> String {
    let mut terms = vec![text.to_string()];
    match text {
        "Casa" => terms.push("House".to_string()),
        "Apartamento" => {
            terms.push("Apartment".to_string());
            terms.push("Condo".to_string());
        }
        "Casa geminada" => terms.push("Townhouse".to_string()),
        "À venda" => {
            terms.push("sale".to_string());
            terms.push("Venda".to_string());
            terms.push("For sale".to_string());
        }
        "Aluguel" => {
            terms.push("rent".to_string());
            terms.push("Locação".to_string());
            terms.push("Aluguel".to_string());
            terms.push("For rent".to_string());
        }
        "Carro/picape" => {
            terms.push("Car/Truck".to_string());
            terms.push("Car or pickup".to_string());
            terms.push("Carro".to_string());
            terms.push("Picape".to_string());
            terms.push("Carro/Caminhão".to_string());
        }
        "Motocicleta" => terms.push("Motorcycle".to_string()),
        "Veículos para esportes" => {
            terms.push("Powersport".to_string());
            terms.push("Powersports".to_string());
        }
        "Trailer" => terms.push("Trailer".to_string()),
        "Reboque" => {
            terms.push("Utility trailer".to_string());
            terms.push("reboque".to_string());
        }
        "Barco" => terms.push("Boat".to_string()),
        "Comercial/industrial" => {
            terms.push("Commercial".to_string());
            terms.push("Industrial".to_string());
        }
        "Excelente" => {
            terms.push("Like new".to_string());
            terms.push("excelente".to_string());
        }
        "Muito bom" => {
            terms.push("Very good".to_string());
            terms.push("muito bom".to_string());
        }
        "Bom" => {
            terms.push("Good".to_string());
            terms.push("bom".to_string());
        }
        "Razoável" => {
            terms.push("Fair".to_string());
            terms.push("razoável".to_string());
        }
        "Ruim" => {
            terms.push("Poor".to_string());
            terms.push("ruim".to_string());
        }
        "Gasolina" => {
            terms.push("Gas".to_string());
            terms.push("Gasoline".to_string());
        }
        "Diesel" => terms.push("Diesel".to_string()),
        "Híbrido" => terms.push("Hybrid".to_string()),
        "Híbrido plug-in" => terms.push("Plug-in hybrid".to_string()),
        "Elétrico" => terms.push("Electric".to_string()),
        "Flex" => terms.push("Flex".to_string()),
        "Cupê" => terms.push("Coupe".to_string()),
        "Sedã" => terms.push("Sedan".to_string()),
        "Hatch" => terms.push("Hatchback".to_string()),
        "SUV" => terms.push("SUV".to_string()),
        "Conversível" => terms.push("Convertible".to_string()),
        "Station wagon" => {
            terms.push("Wagon".to_string());
            terms.push("Station".to_string());
        }
        "Minivan" => terms.push("Minivan".to_string()),
        "Carro compacto" => {
            terms.push("Compact".to_string());
            terms.push("Compact car".to_string());
        }
        "Outro" => terms.push("Other".to_string()),
        _ => {}
    }

    let conditions: Vec<String> = terms
        .into_iter()
        .map(|t| format!("contains(., '{}')", t))
        .collect();

    format!("//*[@role='option'][{}]", conditions.join(" or "))
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
        for _ in 1..=40 {
            if let Ok(el) = self.find_element(selector).await {
                return Ok(el);
            }
            sleep(Duration::from_millis(750)).await;
        }

        // Captura a URL atual para diagnóstico
        let current_url = self
            .evaluate("window.location.href")
            .await
            .ok()
            .and_then(|v| v.into_value::<String>().ok())
            .unwrap_or_else(|| "(não foi possível obter a URL)".to_string());

        logger::error(&format!(
            "Elemento não encontrado após 30s: {} | Página atual: {}",
            selector, current_url
        ));

        Err(DomainError::AutomationError(format!(
            "Elemento não carregou na tela: {}",
            selector
        )))
    }

    async fn wait_for_xpath(&self, xpath: &str) -> Result<Element, DomainError> {
        for _ in 1..=40 {
            if let Ok(el) = self.find_xpath(xpath).await {
                return Ok(el);
            }
            sleep(Duration::from_millis(750)).await;
        }

        // Captura a URL atual para diagnóstico
        let current_url = self
            .evaluate("window.location.href")
            .await
            .ok()
            .and_then(|v| v.into_value::<String>().ok())
            .unwrap_or_else(|| "(não foi possível obter a URL)".to_string());

        logger::error(&format!(
            "XPath não encontrado após 30s: {} | Página atual: {}",
            xpath, current_url
        ));

        Err(DomainError::AutomationError(format!(
            "XPath não carregou na tela: {}",
            xpath
        )))
    }

    async fn click_option(&self, text: &str) -> Result<(), DomainError> {
        let xpath = get_option_xpath(text);
        let el = self.wait_for_xpath(&xpath).await?;
        if let Err(_) = el.click().await {
            // Fallback to JS click
            let click_js = format!(
                r#"(function() {{
                    var el = document.evaluate({:?}, document, null, XPathResult.FIRST_ORDERED_NODE_TYPE, null).singleNodeValue;
                    if (el) {{
                        el.click();
                        return true;
                    }}
                    return false;
                }})()"#,
                xpath
            );
            let _ = self.evaluate(click_js).await;
        }
        sleep(Duration::from_millis(300)).await;
        Ok(())
    }

    async fn focus_and_type(&self, xpath: &str, value: &str) -> Result<(), DomainError> {
        let el = self.wait_for_xpath(xpath).await?;
        if let Err(_) = el.click().await {
            // Fallback: Tenta focar e clicar via JS
            let focus_js = format!(
                r#"(function() {{
                    var el = document.evaluate({:?}, document, null, XPathResult.FIRST_ORDERED_NODE_TYPE, null).singleNodeValue;
                    if (el) {{
                        el.focus();
                        el.click();
                        return true;
                    }}
                    return false;
                }})()"#,
                xpath
            );
            let _ = self.evaluate(focus_js).await;
        }

        let js = format!(
            r#"(function() {{
                var el = document.evaluate({:?}, document, null,
                    XPathResult.FIRST_ORDERED_NODE_TYPE, null).singleNodeValue;
                if (!el) return false;
                el.focus();
                // 1. Tenta usar insertText para simular digitação real
                var success = document.execCommand('insertText', false, {:?});
                if (!success || el.value !== {:?}) {{
                    // 2. Fallback caso execCommand falhe ou não atualize o valor
                    el.value = {:?};
                    el.dispatchEvent(new Event('input', {{ bubbles: true }}));
                    el.dispatchEvent(new Event('change', {{ bubbles: true }}));
                }}
                return true;
            }})()"#,
            xpath, value, value, value
        );

        let ok = self
            .evaluate(js)
            .await
            .map_err(|e| {
                DomainError::AutomationError(format!(
                    "Falha ao executar script de digitação ({}): {}",
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
        if let Err(_) = el.click().await {
            // Fallback to JS click
            let click_js = format!(
                r#"(function() {{
                    var el = document.evaluate({:?}, document, null, XPathResult.FIRST_ORDERED_NODE_TYPE, null).singleNodeValue;
                    if (el) {{
                        el.click();
                        return true;
                    }}
                    return false;
                }})()"#,
                xpath
            );
            let _ = self.evaluate(click_js).await;
        }
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
        Self {
            browser: Some(browser),
        }
    }

    /// Gracefully close the browser, giving Chrome time to flush cookies and
    /// session data to disk. This is critical for Facebook login persistence.
    async fn close(mut self) {
        if let Some(mut browser) = self.browser.take() {
            // 1. Send the close command (graceful shutdown)
            let _ = browser.close().await;
            // 2. Wait for the process to fully exit so cookies are flushed
            let _ = browser.wait().await;
            // 3. Extra safety delay to ensure disk writes complete
            sleep(Duration::from_secs(1)).await;
        }
    }
}

impl Drop for BrowserGuard {
    fn drop(&mut self) {
        if let Some(mut browser) = self.browser.take() {
            // Fallback: if close() wasn't called explicitly, try graceful
            // close in a spawned task. This is less reliable than calling
            // close() explicitly but better than kill().
            tokio::task::spawn(async move {
                let _ = browser.close().await;
                let _ = browser.wait().await;
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
            logger::error(&format!("Erro ao iniciar Google Chrome: {:?}", e));
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
                .map_err(|e| {
                    DomainError::AutomationError(format!(
                        "Falha ao construir configuração headless do browser: {}",
                        e
                    ))
                })?,
        )
        .await
        .map_err(|e| {
            logger::error(&format!("Erro ao iniciar Google Chrome Headless: {:?}", e));
            DomainError::AutomationError(
                "Google Chrome não foi encontrado no sistema ou falhou ao iniciar em modo oculto."
                    .to_string(),
            )
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
                p.goto(url).await.map_err(|e| {
                    DomainError::AutomationError(format!("Falha ao navegar para a página: {}", e))
                })?;
                p
            }
            None => browser.new_page(url).await.map_err(|e| {
                DomainError::AutomationError(format!("Falha ao criar nova página: {}", e))
            })?,
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
        sanitize_client_id(&client_id)?;
        // ── Seletores XPath (Português / Inglês / Espanhol) ───────────────
        const XPATH_MODEL_DROPDOWN: &str = "//label[@role='combobox'][contains(., 'venda ou locação') or contains(., 'Home for sale or rent') or contains(., 'Home for sale') or contains(., 'Property for rent') or contains(., 'Property for sale') or contains(., 'Listing type') or contains(., 'Alquiler')]";
        const XPATH_CATEGORY_DROPDOWN: &str = "//label[@role='combobox'][contains(., 'Tipo de imóvel') or contains(., 'Home type') or contains(., 'Property type') or contains(., 'Tipo de propiedad')]";
        const XPATH_PARKING_DROPDOWN: &str = "//label[@role='combobox'][contains(., 'Vagas de estacionamento') or contains(., 'Parking spaces') or contains(., 'Parking') or contains(., 'Plazas de aparcamiento') or contains(., 'Estacionamiento')]";
        const XPATH_BEDROOM_INPUT: &str = "//span[contains(., 'Número de quartos') or contains(., 'Number of bedrooms') or contains(., 'Bedrooms') or contains(., 'Habitaciones') or contains(., 'Quartos')]/following::input[1]";
        const XPATH_BATHROOM_INPUT: &str = "//span[contains(., 'Número de banheiros') or contains(., 'Number of bathrooms') or contains(., 'Bathrooms') or contains(., 'Baños') or contains(., 'Banheiros')]/following::input[1]";
        const XPATH_PRICE_INPUT: &str = "//span[contains(., 'Preço') or contains(., 'Price') or contains(., 'Precio')]/following::input[1]";
        const XPATH_ADDRESS_INPUT: &str = "//input[@role='combobox'][@aria-autocomplete='list'][not(contains(@aria-label, 'Pesquisar'))][not(contains(@aria-label, 'Search'))]";
        const XPATH_DESCRIPTION_TEXTAREA: &str = "//span[contains(., 'Descrição do imóvel') or contains(., 'Descrição') or contains(., 'Property description') or contains(., 'Description') or contains(., 'Descripción')]/following::textarea[1]";
        const XPATH_METER_INPUT: &str = "//span[contains(., 'Metros quadrados') or contains(., 'Área útil') or contains(., 'Square feet') or contains(., 'Square meters') or contains(., 'Metros cuadrados')]/following::input[1]";
        const XPATH_TAX_INPUT: &str = "//span[contains(., 'Imposto') or contains(., 'Tax') or contains(., 'Impuesto')]/following::input[1]";
        const XPATH_CONDOMINIUM_INPUT: &str = "//span[contains(., 'Condomínio') or contains(., 'Condo') or contains(., 'HOA fee') or contains(., 'HOA') or contains(., 'Condominio')]/following::input[1]";

        let url = "https://www.facebook.com/marketplace/create/rental";

        let browser = Self::launch_browser(client_id.as_str()).await?;
        let guard = BrowserGuard::new(browser);
        let page = Self::get_or_create_page(guard.browser.as_ref().ok_or(DomainError::AutomationError("Browser not available".to_string()))?, url).await?;

        // Forçar idioma português no Facebook via cookie de locale.
        // O parâmetro ?locale=pt_BR na URL não funciona de forma confiável
        // porque o Facebook prioriza a configuração de idioma da conta.
        // O cookie força o servidor a responder em português.
        page.evaluate(
            r#"document.cookie = "locale=pt_BR; domain=.facebook.com; path=/; max-age=31536000; SameSite=None; Secure";"#
        ).await.ok();
        page.evaluate("window.location.reload()").await.ok();
        logger::info("Forçando idioma português no Facebook...");
        sleep(Duration::from_secs(4)).await;

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

        // Verificar se o usuário está logado antes de continuar
        sleep(Duration::from_secs(3)).await;
        let current_url = page
            .evaluate("window.location.href")
            .await
            .ok()
            .and_then(|v| v.into_value::<String>().ok())
            .unwrap_or_default();

        if current_url.contains("login") || current_url.contains("checkpoint") {
            logger::error(&format!(
                "Usuário não está logado no Facebook. URL atual: {}",
                current_url
            ));
            guard.close().await;
            return Err(DomainError::AutomationError(
                "Você precisa estar logado no Facebook antes de publicar. Use a opção 'Entrar' primeiro.".to_string(),
            ));
        }

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

        // Aguardar o formulário carregar completamente após upload das imagens
        logger::info("Fotos enviadas, aguardando formulário carregar...");
        sleep(Duration::from_secs(5)).await;

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
                "A publicação falhou, o tempo esgotou ou a janela foi fechada prematuramente."
                    .to_string(),
            ));
        }

        Ok(())
    }

    async fn add_vehicle(&self, entity: Vehicle, client_id: String) -> Result<(), DomainError> {
        sanitize_client_id(&client_id)?;
        // ── Seletores XPath (Português / Inglês / Espanhol) ───────────────
        const XPATH_TYPE_DROPDOWN: &str = "//label[@role='combobox'][contains(., 'Tipo de veículo') or contains(., 'Vehicle type') or contains(., 'Type') or contains(., 'Tipo de vehículo')]";
        const XPATH_YEAR_DROPDOWN: &str = "//label[@role='combobox'][contains(., 'Ano') or contains(., 'Year') or contains(., 'Año')]";
        const XPATH_MAKE_DROPDOWN: &str = "//label[@role='combobox'][contains(., 'Fabricante') or contains(., 'Make') or contains(., 'Marca')]";
        const XPATH_MAKE_INPUT: &str = "//span[contains(., 'Fabricante') or contains(., 'Make') or contains(., 'Marca')]/following::input[1]";
        const XPATH_MODEL_INPUT: &str = "//span[contains(., 'Modelo') or contains(., 'Model')]/following::input[1]";
        const XPATH_MILEAGE_INPUT: &str = "//span[contains(., 'Quilometragem') or contains(., 'Mileage') or contains(., 'Kilometraje') or contains(., 'Odometer')]/following::input[1]";
        const XPATH_PRICE_INPUT: &str = "//span[contains(., 'Preço') or contains(., 'Price') or contains(., 'Precio')]/following::input[1]";
        const XPATH_BODYSTYLE_DROPDOWN: &str = "//label[@role='combobox'][contains(., 'Estilo da carroceria') or contains(., 'Body style') or contains(., 'Body Style') or contains(., 'Carrocería')]";
        const XPATH_CONDITION_DROPDOWN: &str = "//label[@role='combobox'][contains(., 'Condição do veículo') or contains(., 'Condição') or contains(., 'Condition') or contains(., 'Condición')]";
        const XPATH_FUEL_DROPDOWN: &str = "//label[@role='combobox'][contains(., 'Tipo de combustível') or contains(., 'Fuel type') or contains(., 'Fuel') or contains(., 'Combustible')]";
        const XPATH_LOCATION_INPUT: &str = "//input[@role='combobox'][@aria-label='Localização' or @aria-label='Location' or @aria-label='Ubicación']";
        const XPATH_DESCRIPTION_TEXTAREA: &str = "//span[contains(., 'Descrição') or contains(., 'Description') or contains(., 'Descripción')]/following::textarea[1]";
        const SEL_PHOTO_INPUT: &str = "input[type='file'][accept*='image']";

        let url = "https://www.facebook.com/marketplace/create/vehicle";

        let browser = Self::launch_browser(client_id.as_str()).await?;
        let guard = BrowserGuard::new(browser);
        let page = Self::get_or_create_page(guard.browser.as_ref().ok_or(DomainError::AutomationError("Browser not available".to_string()))?, url).await?;

        // Forçar idioma português no Facebook via cookie de locale.
        page.evaluate(
            r#"document.cookie = "locale=pt_BR; domain=.facebook.com; path=/; max-age=31536000; SameSite=None; Secure";"#
        ).await.ok();
        page.evaluate("window.location.reload()").await.ok();
        logger::info("Forçando idioma português no Facebook...");
        sleep(Duration::from_secs(4)).await;

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

        // Verificar se o usuário está logado antes de continuar
        sleep(Duration::from_secs(3)).await;
        let current_url = page
            .evaluate("window.location.href")
            .await
            .ok()
            .and_then(|v| v.into_value::<String>().ok())
            .unwrap_or_default();

        if current_url.contains("login") || current_url.contains("checkpoint") {
            logger::error(&format!(
                "Usuário não está logado no Facebook. URL atual: {}",
                current_url
            ));
            guard.close().await;
            return Err(DomainError::AutomationError(
                "Você precisa estar logado no Facebook antes de publicar. Use a opção 'Entrar' primeiro.".to_string(),
            ));
        }

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

        // Aguardar o formulário carregar completamente após upload das imagens
        logger::info("Fotos enviadas, aguardando formulário carregar...");
        sleep(Duration::from_secs(5)).await;

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
                "A publicação falhou, o tempo esgotou ou a janela foi fechada prematuramente."
                    .to_string(),
            ));
        }

        Ok(())
    }

    async fn signin(&self, client_id: String) -> Result<(), DomainError> {
        sanitize_client_id(&client_id)?;
        let browser = Self::launch_browser(client_id.as_str()).await?;
        let guard = BrowserGuard::new(browser);
        let page = Self::get_or_create_page(
            guard.browser.as_ref().ok_or(DomainError::AutomationError("Browser not available".to_string()))?,
            "https://www.facebook.com/login?locale=pt_BR",
        )
        .await?;

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
        sanitize_client_id(&client_id)?;
        let browser = Self::launch_browser_headless(client_id.as_str()).await?;
        let guard = BrowserGuard::new(browser);
        let page = Self::get_or_create_page(
            guard.browser.as_ref().ok_or(DomainError::AutomationError("Browser not available".to_string()))?,
            "https://www.facebook.com/?locale=pt_BR",
        )
        .await?;

        sleep(Duration::from_secs(6)).await;

        if page.find_element(SEL_FACEBOOK_LOGGED_IN).await.is_err() {
            guard.close().await;
            return Ok(());
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
        sanitize_client_id(&client_id)?;
        let browser = Self::launch_browser_headless(client_id.as_str()).await?;
        let guard = BrowserGuard::new(browser);

        let page = Self::get_or_create_page(
            guard.browser.as_ref().ok_or(DomainError::AutomationError("Browser not available".to_string()))?,
            "https://www.facebook.com/?locale=pt_BR",
        )
        .await?;

        sleep(Duration::from_secs(6)).await;

        let is_logged_in = page.find_element(SEL_FACEBOOK_LOGGED_IN).await.is_ok();

        guard.close().await;
        Ok(is_logged_in)
    }
}
