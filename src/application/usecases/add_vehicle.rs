use std::sync::Arc;

use crate::{
    application::error::UseCasesError,
    domain::{
        repositories::image::ImageRepository,
        services::{
            vehicle::VehicleService, webscraping::marketplace::WebscrapingMarketplaceService,
        },
    },
};

pub struct AddVehicleUseCase<
    _ImageRepository: ImageRepository,
    _WebscrapingMarketplaceService: WebscrapingMarketplaceService,
    _VehicleService: VehicleService,
> {
    image_repository: Arc<_ImageRepository>,
    webscraping_marketplace_service: Arc<_WebscrapingMarketplaceService>,
    vehicle_service: Arc<_VehicleService>,
}

impl<
    _ImageRepository: ImageRepository,
    _WebscrapingMarketplaceService: WebscrapingMarketplaceService,
    _VehicleService: VehicleService,
> AddVehicleUseCase<_ImageRepository, _WebscrapingMarketplaceService, _VehicleService>
{
    pub fn new(
        image_repository: Arc<_ImageRepository>,
        webscraping_marketplace_service: Arc<_WebscrapingMarketplaceService>,
        vehicle_service: Arc<_VehicleService>,
    ) -> Self {
        Self {
            image_repository,
            webscraping_marketplace_service,
            vehicle_service,
        }
    }

    pub async fn handle(
        &self,
        url: String,
        token: String,
        client_id: String,
    ) -> Result<(), UseCasesError> {
        let mut response = self.vehicle_service.get(url, token).await?;

        let images = self.image_repository.add(response.image().clone()).await;

        response.set_image(images);

        self.webscraping_marketplace_service
            .add_vehicle(response, client_id)
            .await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::application::tests::{
        repositories::image::InMemoryImageRepository,
        services::{
            vehicle::InMemoryVehicleService,
            webscraping::marketplace::InMemoryWebscrapingMarketplaceService,
        },
    };

    use super::*;

    #[tokio::test]
    async fn success() {
        let image_repository = Arc::new(InMemoryImageRepository::new());
        let webscraping_marketplace_service =
            Arc::new(InMemoryWebscrapingMarketplaceService::new());
        let vehicle_service = Arc::new(InMemoryVehicleService::new());

        let usecase = AddVehicleUseCase::new(
            image_repository,
            webscraping_marketplace_service,
            vehicle_service,
        );

        let url = "https://example.com".to_string();
        let token = "asdASD123".to_string();
        let client_id = "123".to_string();

        let response = usecase.handle(url, token, client_id).await;

        assert!(response.is_ok());
    }
}
