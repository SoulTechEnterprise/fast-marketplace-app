use std::sync::Arc;

use crate::{
    application::error::UseCasesError,
    domain::{
        entities::property::Property, repositories::image::ImageRepository,
        services::webscraping::marketplace::WebscrapingMarketplaceService,
    },
};

pub struct AddPropertyUseCase<
    _ImageRepository: ImageRepository,
    _WebscrapingMarketplaceService: WebscrapingMarketplaceService,
> {
    image_repository: Arc<_ImageRepository>,
    webscraping_marketplace_service: Arc<_WebscrapingMarketplaceService>,
}

impl<
    _ImageRepository: ImageRepository,
    _WebscrapingMarketplaceService: WebscrapingMarketplaceService,
> AddPropertyUseCase<_ImageRepository, _WebscrapingMarketplaceService>
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
        mut property: Property,
    ) -> Result<(), UseCasesError> {
        let images = self.image_repository.add(property.image().clone()).await;

        property.set_image(images);

        self.webscraping_marketplace_service
            .add_property(property, client_id)
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
        domain::entities::models::property::{category::Category, model::Model},
    };

    use super::*;

    #[tokio::test]
    async fn success() {
        let image_repository = Arc::new(InMemoryImageRepository::new());
        let webscraping_marketplace_service =
            Arc::new(InMemoryWebscrapingMarketplaceService::new());

        let usecase = AddPropertyUseCase::new(image_repository, webscraping_marketplace_service);

        let client_id = "123".to_string();

        let image = vec!["http://example.com/image/1".to_string()];
        let model = Model::Sale;
        let category = Category::House;
        let bedroom = 2;
        let bathroom = 2;
        let price = 1000000;
        let address = "A street example".to_string();
        let description = "An example description".to_string();
        let meter = 250;
        let tax = 100;
        let condominium = 300;
        let parking = 3;

        let property = Property::new(
            image,
            model,
            category,
            bedroom,
            bathroom,
            price,
            address,
            description,
            meter,
            tax,
            condominium,
            parking,
        );

        let response = usecase.handle(client_id, property).await;

        assert!(response.is_ok());
    }
}
