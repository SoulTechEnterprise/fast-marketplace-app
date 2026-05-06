use crate::domain::entities::models::vehicle::category::Category;

#[derive(Clone, Debug)]
pub struct Vehicle {
    category: Category,
    image: Vec<String>,
    address: String,
    year: u16,
    manufacturer: String,
    model: String,
    price: u32,
    description: String,
}

impl Vehicle {
    pub fn new(
        category: Category,
        image: Vec<String>,
        address: String,
        year: u16,
        manufacturer: String,
        model: String,
        price: u32,
        description: String,
    ) -> Self {
        Self {
            category,
            image,
            address,
            year,
            manufacturer,
            model,
            price,
            description,
        }
    }

    pub fn category(&self) -> &Category {
        &self.category
    }

    pub fn image(&self) -> &Vec<String> {
        &self.image
    }

    pub fn address(&self) -> &String {
        &self.address
    }

    pub fn year(&self) -> u16 {
        self.year
    }

    pub fn manufacturer(&self) -> &String {
        &self.manufacturer
    }

    pub fn model(&self) -> &String {
        &self.model
    }

    pub fn price(&self) -> u32 {
        self.price
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
    pub price: String,
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
        price: String,
        description: String,
    ) -> Self {
        Self {
            category,
            image,
            address,
            year,
            manufacturer,
            model,
            price,
            description,
        }
    }
}
