use serde::{Deserialize, Serialize};

use crate::domain::entities::models::property::{category::Category, model::Model};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Property {
    image: Vec<String>,
    model: Model,
    category: Category,
    bedroom: u8,
    bathroom: u8,
    price: u32,
    address: String,
    description: String,
    meter: u16,
    tax: u16,
    condominium: u16,
    parking: u8,
}

impl Property {
    pub fn new(
        image: Vec<String>,
        model: Model,
        category: Category,
        bedroom: u8,
        bathroom: u8,
        price: u32,
        address: String,
        description: String,
        meter: u16,
        tax: u16,
        condominium: u16,
        parking: u8,
    ) -> Self {
        Self {
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
        }
    }

    pub fn image(&self) -> &Vec<String> {
        &self.image
    }

    pub fn set_image(&mut self, image: Vec<String>) -> () {
        self.image = image
    }

    pub fn model(&self) -> &Model {
        &self.model
    }

    pub fn category(&self) -> &Category {
        &self.category
    }

    pub fn bedroom(&self) -> u8 {
        self.bedroom
    }

    pub fn bathroom(&self) -> u8 {
        self.bathroom
    }

    pub fn price(&self) -> u32 {
        self.price
    }

    pub fn address(&self) -> &String {
        &self.address
    }

    pub fn description(&self) -> &String {
        &self.description
    }

    pub fn meter(&self) -> u16 {
        self.meter
    }

    pub fn tax(&self) -> u16 {
        self.tax
    }

    pub fn condominium(&self) -> u16 {
        self.condominium
    }

    pub fn parking(&self) -> u8 {
        self.parking
    }
}

#[derive(Clone, Debug)]
pub struct PropertyXPath {
    pub image: String,
    pub model: String,
    pub category: String,
    pub bedroom: String,
    pub bathroom: String,
    pub price: String,
    pub address: String,
    pub description: String,
    pub meter: String,
    pub tax: String,
    pub condominium: String,
    pub parking: String,
}

impl PropertyXPath {
    pub fn new(
        image: String,
        model: String,
        category: String,
        bedroom: String,
        bathroom: String,
        price: String,
        address: String,
        description: String,
        meter: String,
        tax: String,
        condominium: String,
        parking: String,
    ) -> Self {
        Self {
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
        }
    }
}
