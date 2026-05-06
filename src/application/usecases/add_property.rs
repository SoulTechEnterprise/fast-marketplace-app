use std::sync::Arc;

use crate::{
    application::error::UseCaseError,
    domain::{
        repositories::image::ImageRepository,
        services::{
            property::PropertyService, webscraping::marketplace::WebscrapingMarketplaceService,
        },
    },
};

pub struct AddPropertyUseCase<
    _ImageRepository: ImageRepository,
    _WebscrapingMarketplaceService: WebscrapingMarketplaceService,
    _PropertyService: PropertyService,
> {
    image_repository: Arc<_ImageRepository>,
    webscraping_marketplace_service: Arc<_WebscrapingMarketplaceService>,
    property_service: Arc<_PropertyService>,
}

impl<
    _ImageRepository: ImageRepository,
    _WebscrapingMarketplaceService: WebscrapingMarketplaceService,
    _PropertyService: PropertyService,
> AddPropertyUseCase<_ImageRepository, _WebscrapingMarketplaceService, _PropertyService>
{
    pub fn new(
        image_repository: Arc<_ImageRepository>,
        webscraping_marketplace_service: Arc<_WebscrapingMarketplaceService>,
        property_service: Arc<_PropertyService>,
    ) -> Self {
        Self {
            image_repository,
            webscraping_marketplace_service,
            property_service,
        }
    }

    pub async fn handle(
        &self,
        url: String,
        token: String,
        client_id: String,
    ) -> Result<(), UseCaseError> {
        let mut response = self.property_service.get(url, token).await?;

        let images = self.image_repository.add(response.image().clone()).await;

        response.set_image(images);

        self.webscraping_marketplace_service
            .add_property(response, client_id)
            .await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::application::tests::{
        repositories::image::InMemoryImageRepository,
        services::{
            property::InMemoryPropertyService,
            webscraping::marketplace::InMemoryWebscrapingMarketplaceService,
        },
    };

    use super::*;

    #[tokio::test]
    async fn success() {
        let image_repository = Arc::new(InMemoryImageRepository::new());
        let webscraping_marketplace_service =
            Arc::new(InMemoryWebscrapingMarketplaceService::new());
        let property_service = Arc::new(InMemoryPropertyService::new());

        let usecase = AddPropertyUseCase::new(
            image_repository,
            webscraping_marketplace_service,
            property_service,
        );

        let url = "https://example.com".to_string();
        let token = "asdASD123".to_string();
        let client_id = "123".to_string();

        let response = usecase.handle(url, token, client_id).await;

        assert!(response.is_ok());
    }
}
