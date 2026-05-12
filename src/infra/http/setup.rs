use std::sync::Arc;

use crate::{
    application::usecases::{
        add_property::AddPropertyUseCase, add_vehicle::AddVehicleUseCase,
        get_marketplace::GetMarketplaceUseCase, signin_marketplace::SignInMarketplaceUseCase,
        signout_marketplace::SignOutMarketplaceUseCase,
    },
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
    pub get_marketplace_usecase: Arc<GetMarketplaceUseCase<FacebookMarketplaceService>>,
    pub signin_marketplace_usecase: Arc<SignInMarketplaceUseCase<FacebookMarketplaceService>>,
    pub signout_marketplace_usecase: Arc<SignOutMarketplaceUseCase<FacebookMarketplaceService>>,
}
