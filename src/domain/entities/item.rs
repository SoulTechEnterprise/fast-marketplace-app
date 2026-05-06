use crate::domain::entities::models::item::{
    availability::Availability, category::Category, condition::Condition, meeting::Meeting,
};

#[derive(Clone, Debug)]
pub struct Item {
    image: Vec<String>,
    title: String,
    price: u32,
    category: Category,
    condition: Condition,
    description: String,
    availability: Availability,
    address: String,
    meeting: Vec<Meeting>,
}

impl Item {
    pub fn new(
        image: Vec<String>,
        title: String,
        price: u32,
        category: Category,
        condition: Condition,
        description: String,
        availability: Availability,
        address: String,
        meeting: Vec<Meeting>,
    ) -> Self {
        Self {
            image,
            title,
            price,
            category,
            condition,
            description,
            availability,
            address,
            meeting,
        }
    }

    pub fn image(&self) -> &Vec<String> {
        &self.image
    }

    pub fn title(&self) -> &String {
        &self.title
    }

    pub fn price(&self) -> u32 {
        self.price
    }

    pub fn category(&self) -> &Category {
        &self.category
    }

    pub fn condition(&self) -> &Condition {
        &self.condition
    }

    pub fn description(&self) -> &String {
        &self.description
    }

    pub fn availability(&self) -> &Availability {
        &self.availability
    }

    pub fn address(&self) -> &String {
        &self.address
    }

    pub fn meeting(&self) -> &Vec<Meeting> {
        &self.meeting
    }
}

#[derive(Clone, Debug)]
pub struct ItemXPath {
    pub image: String,
    pub title: String,
    pub price: String,
    pub category: String,
    pub condition: String,
    pub description: String,
    pub availability: String,
    pub address: String,
    pub meeting: String,
}

impl ItemXPath {
    pub fn new(
        image: String,
        title: String,
        price: String,
        category: String,
        condition: String,
        description: String,
        availability: String,
        address: String,
        meeting: String,
    ) -> Self {
        Self {
            image,
            title,
            price,
            category,
            condition,
            description,
            availability,
            address,
            meeting,
        }
    }
}
