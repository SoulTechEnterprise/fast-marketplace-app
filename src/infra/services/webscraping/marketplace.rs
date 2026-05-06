use async_trait::async_trait;
use chromiumoxide::cdp::browser_protocol::dom::SetFileInputFilesParams;
use chromiumoxide::{
    Element, Page,
    browser::{Browser, BrowserConfig},
};
use futures::StreamExt;
use std::path::PathBuf;
use tokio::sync::Mutex;
use tokio::time::{Duration, sleep};

use crate::domain::{
    entities::{item::Item, property::Property, vehicle::Vehicle},
    services::{error::DomainError, webscraping::marketplace::WebscrapingMarketplaceService},
};

// ── XPath estáveis (baseados no texto do label, não em ids dinâmicos) ─────────

const XPATH_MODEL_DROPDOWN: &str = "//label[@role='combobox'][contains(., 'venda ou locação')]";
const XPATH_CATEGORY_DROPDOWN: &str = "//label[@role='combobox'][contains(., 'Tipo de imóvel')]";
const XPATH_PARKING_DROPDOWN: &str =
    "//label[@role='combobox'][contains(., 'Vagas de estacionamento')]";
const XPATH_BEDROOM_INPUT: &str = "//span[contains(., 'Número de quartos')]/following::input[1]";
const XPATH_BATHROOM_INPUT: &str = "//span[contains(., 'Número de banheiros')]/following::input[1]";
const XPATH_PRICE_INPUT: &str = "//span[contains(., 'Preço')]/following::input[1]";
const XPATH_ADDRESS_INPUT: &str =
    "//input[@role='combobox'][@aria-autocomplete='list'][not(contains(@aria-label, 'Pesquisar'))]";
const XPATH_DESCRIPTION_TEXTAREA: &str =
    "//span[contains(., 'Descrição do imóvel')]/following::textarea[1]";
const XPATH_METER_INPUT: &str = "//span[contains(., 'Metros quadrados')]/following::input[1]";
const XPATH_TAX_INPUT: &str = "//span[contains(., 'Imposto')]/following::input[1]";
const XPATH_CONDOMINIUM_INPUT: &str = "//span[contains(., 'Condomínio')]/following::input[1]";

const SEL_PHOTO_INPUT: &str = "input[type='file']";
const SEL_FACEBOOK_LOGGED_IN: &str = "div[aria-label='Facebook']";

const SEL_FACEBOOK_TRUST_DEVICE: &str = "div[data-testid='save-device-button'], \
                                          button[name='save_device'], \
                                          div[aria-label='Salvar dispositivo'], \
                                          .__7n5 button"; // fallback genérico

// ── Helpers ───────────────────────────────────────────────────────────────────

fn model_to_label(model: &crate::domain::entities::models::property::model::Model) -> &'static str {
    match model {
        crate::domain::entities::models::property::model::Model::Sale => "À venda",
        crate::domain::entities::models::property::model::Model::Rent => "Aluguel",
    }
}

fn category_to_label(
    category: &crate::domain::entities::models::property::category::Category,
) -> &'static str {
    match category {
        crate::domain::entities::models::property::category::Category::Apartment => "Apartamento",
        crate::domain::entities::models::property::category::Category::House => "Casa",
    }
}

fn profile_dir(client_id: &str) -> PathBuf {
    #[cfg(target_os = "windows")]
    let base = dirs::data_local_dir().unwrap_or_else(|| PathBuf::from("."));
    #[cfg(not(target_os = "windows"))]
    let base = dirs::data_dir().unwrap_or_else(|| PathBuf::from("."));

    let dir = base
        .join("fast-marketplace")
        .join("chrome-profiles")
        .join(client_id);
    std::fs::create_dir_all(&dir).ok();
    dir
}

// ── Extension Trait para Page ─────────────────────────────────────────────────

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
        eprintln!("❌ Não encontrado: {}", selector);
        // 👇 ERRO DETALHADO
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
        eprintln!("❌ Não encontrado: {}", xpath);
        // 👇 ERRO DETALHADO
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
            eprintln!("❌ focus_and_type falhou: {}", xpath);
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

// ── Serviço ───────────────────────────────────────────────────────────────────

pub struct FacebookMarketplaceService {
    browser: Mutex<Option<Browser>>,
    page: Mutex<Option<Page>>,
}

impl FacebookMarketplaceService {
    pub fn new() -> Self {
        Self {
            browser: Mutex::new(None),
            page: Mutex::new(None),
        }
    }

    async fn launch_browser(client_id: &str) -> Result<Browser, DomainError> {
        let (browser, mut handler) = Browser::launch(
            BrowserConfig::builder()
                .with_head()
                .user_data_dir(profile_dir(client_id))
                .arg("--start-maximized")
                .arg("--disable-infobars")
                .arg("--disable-notifications")
                .arg("--disable-blink-features=AutomationControlled")
                // Anti-detecção
                .arg("--no-sandbox")
                .arg("--disable-dev-shm-usage")
                .arg("--disable-web-security")
                .arg("--disable-features=IsolateOrigins,site-per-process")
                .arg("--allow-running-insecure-content")
                .arg("--disable-site-isolation-trials")
                // Remove "Chrome está sendo controlado por software automatizado"
                .arg("--excludeSwitches=enable-automation")
                .arg("--useAutomationExtension=false")
                .arg("--user-agent=Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/136.0.0.0 Safari/537.36")
                .build()
                .map_err(|e| {
                    eprintln!("❌ BrowserConfig: {:?}", e);
                    DomainError::NotFound
                })?,
        )
        .await
        .map_err(|e| {
            eprintln!("❌ Browser launch: {:?}", e);
            DomainError::NotFound
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

    /// Abre o browser e navega para o marketplace.
    /// Chame antes de qualquer `add_*`.
    pub async fn open(&self, url: &str, client_id: String) -> Result<(), DomainError> {
        let browser = Self::launch_browser(client_id.as_str()).await?;
        let page = browser
            .new_page(url)
            .await
            .map_err(|_| DomainError::NotFound)?;

        *self.browser.lock().await = Some(browser);
        *self.page.lock().await = Some(page);

        Ok(())
    }

    /// Fecha o browser. Chame quando terminar.
    pub async fn close(&self) {
        if let Some(mut browser) = self.browser.lock().await.take() {
            let _ = browser.close().await;
        }
        *self.page.lock().await = None;
    }

    /// Abre o Facebook e aguarda o login manual. Salva a sessão no perfil.
    /// Chame apenas uma vez na primeira execução.
    pub async fn login(&self, client_id: String) -> Result<(), DomainError> {
        let mut browser = Self::launch_browser(client_id.as_str()).await?;
        let page = browser
            .new_page("https://www.facebook.com/login")
            .await
            .map_err(|_| DomainError::NotFound)?;

        for _ in 0..240 {
            sleep(Duration::from_secs(2)).await;
            if let Ok(js_result) = page.evaluate("window.location.href").await {
                if let Ok(current_url) = js_result.into_value::<String>() {
                    let is_out_of_login = !current_url.contains("login")
                        && !current_url.contains("checkpoint")
                        && !current_url.contains("two_factor")
                        && !current_url.contains("two-factor")
                        && !current_url.contains("save-device")
                        && !current_url.contains("trust");

                    if is_out_of_login && page.find_element(SEL_FACEBOOK_LOGGED_IN).await.is_ok() {
                        // Verifica se ainda está mostrando o prompt "Confiar nesse dispositivo"
                        let trust_prompt_visible =
                            page.find_element(SEL_FACEBOOK_TRUST_DEVICE).await.is_ok();

                        if trust_prompt_visible {
                            println!(
                                "⏳ Aguardando decisão do prompt 'Confiar nesse dispositivo'..."
                            );
                            continue;
                        }

                        println!("✅ Login detectado! Sessão salva em profiles/{client_id}");
                        sleep(Duration::from_secs(4)).await;
                        let _ = browser.close().await;
                        return Ok(());
                    }
                }
            }
        }

        eprintln!("❌ Timeout de login.");
        let _ = browser.close().await;
        Err(DomainError::NotFound)
    }

    async fn get_page<'a>(
        guard: &'a tokio::sync::MutexGuard<'a, Option<Page>>,
    ) -> Result<&'a Page, DomainError> {
        guard.as_ref().ok_or_else(|| {
            eprintln!("❌ Browser não aberto. Chame open() primeiro.");
            DomainError::NotFound
        })
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
        let url = "https://www.facebook.com/marketplace/create/rental".to_string();

        // 👇 Substituímos o unwrap() por ? para repassar o erro corretamente
        self.open(&url, client_id).await?;

        let guard = self.page.lock().await;
        let page = Self::get_page(&guard).await?;

        page.evaluate(
            r#"
            Object.defineProperty(navigator, 'webdriver', {
                get: () => undefined,
                configurable: true
            });

            // Remove rastros do chrome driver
            delete window.cdc_adoQpoasnfa76pfcZLmcfl_Array;
            delete window.cdc_adoQpoasnfa76pfcZLmcfl_Promise;
            delete window.cdc_adoQpoasnfa76pfcZLmcfl_Symbol;

            // Simula plugins reais
            Object.defineProperty(navigator, 'plugins', {
                get: () => [1, 2, 3, 4, 5],
            });

            // Simula linguagens reais
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

        // ── Fotos ──
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

        // ── Dropdowns ── (Os erros já vêm mastigados do trait PageExt)
        page.select_dropdown(XPATH_MODEL_DROPDOWN, model_to_label(entity.model()))
            .await?;
        page.select_dropdown(
            XPATH_CATEGORY_DROPDOWN,
            category_to_label(entity.category()),
        )
        .await?;

        // ── Inputs de texto ──
        page.focus_and_type(XPATH_BEDROOM_INPUT, &entity.bedroom().to_string())
            .await?;
        page.focus_and_type(XPATH_BATHROOM_INPUT, &entity.bathroom().to_string())
            .await?;
        page.focus_and_type(XPATH_PRICE_INPUT, &entity.price().to_string())
            .await?;

        // ── Endereço com autocomplete ──
        page.focus_and_type(XPATH_ADDRESS_INPUT, entity.address())
            .await?;
        sleep(Duration::from_millis(800)).await;

        // No caso do endereço, se não achar a sugestão a gente só não clica, então não estouramos erro aqui
        if let Ok(el) = page.find_xpath("//*[@role='option'][1]").await {
            let _ = el.click().await;
        }

        // ── Demais campos ──
        page.focus_and_type(XPATH_DESCRIPTION_TEXTAREA, entity.description())
            .await?;
        page.focus_and_type(XPATH_METER_INPUT, &entity.meter().to_string())
            .await?;
        page.focus_and_type(XPATH_TAX_INPUT, &entity.tax().to_string())
            .await?;
        page.focus_and_type(XPATH_CONDOMINIUM_INPUT, &entity.condominium().to_string())
            .await?;

        // ── Vagas ──
        page.select_dropdown(XPATH_PARKING_DROPDOWN, &entity.parking().to_string())
            .await?;

        let max_attempts = 240;
        for _ in 0..max_attempts {
            sleep(Duration::from_secs(2)).await;

            if let Ok(js_result) = page.evaluate("window.location.href").await {
                if let Ok(current_url) = js_result.into_value::<String>() {
                    if current_url.contains("marketplace/you/selling") {
                        break;
                    }
                }
            }
        }

        drop(guard);

        let _ = self.close().await;

        Ok(())
    }

    async fn add_vehicle(&self, _entity: Vehicle) -> Result<(), DomainError> {
        todo!()
    }

    async fn add_item(&self, _entity: Item) -> Result<(), DomainError> {
        todo!()
    }
}
