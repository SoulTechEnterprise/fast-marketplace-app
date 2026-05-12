use std::sync::Arc;

use crate::{
    application::error::UseCasesError,
    domain::services::webscraping::marketplace::WebscrapingMarketplaceService,
};

pub struct SignInMarketplaceUseCase<_WebscrapingMarketplaceService: WebscrapingMarketplaceService> {
    webscraping_marketplace_service: Arc<_WebscrapingMarketplaceService>,
}

impl<_WebscrapingMarketplaceService: WebscrapingMarketplaceService>
    SignInMarketplaceUseCase<_WebscrapingMarketplaceService>
{
    pub fn new(webscraping_marketplace_service: Arc<_WebscrapingMarketplaceService>) -> Self {
        Self {
            webscraping_marketplace_service,
        }
    }

    pub async fn handle(&self, client_id: String) -> Result<(), UseCasesError> {
        self.webscraping_marketplace_service
            .signin(client_id)
            .await?;

        Ok(())
    }
}
