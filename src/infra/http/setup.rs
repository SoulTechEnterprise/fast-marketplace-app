use std::sync::Arc;

use crate::{
    application::usecases::{add_property::AddPropertyUseCase, add_vehicle::AddVehicleUseCase},
    infra::{
        repositories::image::ImageRepositoryImpl,
        services::{
            property::PropertyServiceApi, vehicle::VehicleServiceApi,
            webscraping::marketplace::FacebookMarketplaceService,
        },
    },
};

#[derive(Clone)]
pub struct AppState {
    pub property_usecase: Arc<
        AddPropertyUseCase<ImageRepositoryImpl, FacebookMarketplaceService, PropertyServiceApi>,
    >,
    pub vehicle_usecase:
        Arc<AddVehicleUseCase<ImageRepositoryImpl, FacebookMarketplaceService, VehicleServiceApi>>,
    pub auth_marketplace: Arc<FacebookMarketplaceService>,
}
