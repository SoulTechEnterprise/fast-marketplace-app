use std::sync::Arc;

use crate::{
    application::error::UseCasesError,
    domain::services::webscraping::marketplace::WebscrapingMarketplaceService,
};

pub struct GetMarketplaceUseCase<_WebscrapingMarketplaceService: WebscrapingMarketplaceService> {
    webscraping_marketplace_service: Arc<_WebscrapingMarketplaceService>,
}

impl<_WebscrapingMarketplaceService: WebscrapingMarketplaceService>
    GetMarketplaceUseCase<_WebscrapingMarketplaceService>
{
    pub fn new(webscraping_marketplace_service: Arc<_WebscrapingMarketplaceService>) -> Self {
        Self {
            webscraping_marketplace_service,
        }
    }

    pub async fn handle(&self, client_id: String) -> Result<bool, UseCasesError> {
        let response = self
            .webscraping_marketplace_service
            .get_account(client_id)
            .await?;

        Ok(response)
    }
}
