use std::sync::Arc;

use crate::{
    application::error::UseCasesError,
    domain::{
        entities::vehicle::Vehicle, repositories::image::ImageRepository,
        services::webscraping::marketplace::WebscrapingMarketplaceService,
    },
};

pub struct AddVehicleUseCase<
    _ImageRepository: ImageRepository,
    _WebscrapingMarketplaceService: WebscrapingMarketplaceService,
> {
    image_repository: Arc<_ImageRepository>,
    webscraping_marketplace_service: Arc<_WebscrapingMarketplaceService>,
}

impl<
    _ImageRepository: ImageRepository,
    _WebscrapingMarketplaceService: WebscrapingMarketplaceService,
> AddVehicleUseCase<_ImageRepository, _WebscrapingMarketplaceService>
{
    pub fn new(
        image_repository: Arc<_ImageRepository>,
        webscraping_marketplace_service: Arc<_WebscrapingMarketplaceService>,
    ) -> Self {
        Self {
            image_repository,
            webscraping_marketplace_service,
        }
    }

    pub async fn handle(
        &self,
        client_id: String,
        mut vehicle: Vehicle,
    ) -> Result<(), UseCasesError> {
        let images = self.image_repository.add(vehicle.image().clone()).await;

        vehicle.set_image(images);

        self.webscraping_marketplace_service
            .add_vehicle(vehicle, client_id)
            .await?;

        self.image_repository.remove().await;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        application::tests::{
            repositories::image::InMemoryImageRepository,
            services::webscraping::marketplace::InMemoryWebscrapingMarketplaceService,
        },
        domain::entities::models::vehicle::{
            bodystyle::BodyStyle, category::Category, condition::Condition, fuel::Fuel,
            manufacturer::Manufacturer,
        },
    };

    use super::*;

    #[tokio::test]
    async fn success() {
        let image_repository = Arc::new(InMemoryImageRepository::new());
        let webscraping_marketplace_service =
            Arc::new(InMemoryWebscrapingMarketplaceService::new());

        let usecase = AddVehicleUseCase::new(image_repository, webscraping_marketplace_service);

        let client_id = "123".to_string();

        let category = Category::CarOrPickup;
        let image = vec!["https://example.com/image/1".to_string()];
        let address = "A street example".to_string();
        let year = 2025;
        let manufacturer = Manufacturer::Adly;
        let model = "".to_string();
        let mileage = 1000;
        let bodystyle = BodyStyle::CompactCar;
        let price = 100000;
        let condition = Condition::Excellent;
        let fuel = Fuel::Flex;
        let description = "An example description".to_string();

        let vehicle = Vehicle::new(
            category,
            image,
            address,
            year,
            manufacturer,
            model,
            mileage,
            bodystyle,
            price,
            condition,
            fuel,
            description,
        );

        let response = usecase.handle(client_id, vehicle).await;

        assert!(response.is_ok());
    }
}
