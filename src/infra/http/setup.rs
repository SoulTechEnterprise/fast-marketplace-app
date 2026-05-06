use std::sync::Arc;

use crate::{
    application::usecases::add_property::AddPropertyUseCase,
    infra::{
        repositories::image::ImageRepositoryImpl,
        services::{
            property::PropertyServiceApi, webscraping::marketplace::FacebookMarketplaceService,
        },
    },
};

#[derive(Clone)]
pub struct AppState {
    pub property_usecase: Arc<
        AddPropertyUseCase<ImageRepositoryImpl, FacebookMarketplaceService, PropertyServiceApi>,
    >,
    pub auth_marketplace: Arc<FacebookMarketplaceService>,
}
