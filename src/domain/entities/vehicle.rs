use serde::{Deserialize, Serialize};

use crate::domain::entities::models::vehicle::{
    bodystyle::BodyStyle, category::Category, condition::Condition, fuel::Fuel,
    manufacturer::Manufacturer,
};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Vehicle {
    category: Category,
    image: Vec<String>,
    address: String,
    year: u16,
    manufacturer: Manufacturer,
    model: String,
    mileage: u32,
    bodystyle: BodyStyle,
    price: u32,
    condition: Condition,
    fuel: Fuel,
    description: String,
}

impl Vehicle {
    pub fn new(
        category: Category,
        image: Vec<String>,
        address: String,
        year: u16,
        manufacturer: Manufacturer,
        model: String,
        mileage: u32,
        bodystyle: BodyStyle,
        price: u32,
        condition: Condition,
        fuel: Fuel,
        description: String,
    ) -> Self {
        Self {
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
        }
    }

    pub fn category(&self) -> &Category {
        &self.category
    }

    pub fn image(&self) -> &Vec<String> {
        &self.image
    }

    pub fn set_image(&mut self, image: Vec<String>) -> () {
        self.image = image
    }

    pub fn address(&self) -> &String {
        &self.address
    }

    pub fn year(&self) -> u16 {
        self.year
    }

    pub fn manufacturer(&self) -> &Manufacturer {
        &self.manufacturer
    }

    pub fn model(&self) -> &String {
        &self.model
    }

    pub fn mileage(&self) -> u32 {
        self.mileage
    }

    pub fn bodystyle(&self) -> &BodyStyle {
        &self.bodystyle
    }

    pub fn price(&self) -> u32 {
        self.price
    }

    pub fn condition(&self) -> &Condition {
        &self.condition
    }

    pub fn fuel(&self) -> &Fuel {
        &self.fuel
    }

    pub fn description(&self) -> &String {
        &self.description
    }
}

#[derive(Clone, Debug)]
pub struct VehicleXPath {
    pub category: String,
    pub image: String,
    pub address: String,
    pub year: String,
    pub manufacturer: String,
    pub model: String,
    pub mileage: String,
    pub bodystyle: String,
    pub price: String,
    pub condition: String,
    pub fuel: String,
    pub description: String,
}

impl VehicleXPath {
    pub fn new(
        category: String,
        image: String,
        address: String,
        year: String,
        manufacturer: String,
        model: String,
        mileage: String,
        bodystyle: String,
        price: String,
        condition: String,
        fuel: String,
        description: String,
    ) -> Self {
        Self {
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
        }
    }
}
