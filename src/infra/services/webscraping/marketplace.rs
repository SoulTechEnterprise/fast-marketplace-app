use async_trait::async_trait;
use chromiumoxide::cdp::browser_protocol::dom::SetFileInputFilesParams;
use chromiumoxide::{
    Element, Page,
    browser::{Browser, BrowserConfig},
};
use futures::StreamExt;
use std::path::PathBuf;
use tokio::sync::Mutex;
use tokio::time::{Duration, sleep};

use crate::domain::entities::models::vehicle::bodystyle::BodyStyle as VehicleBodyStyle;
use crate::domain::entities::models::vehicle::category::Category as VehicleCategory;
use crate::domain::entities::models::vehicle::condition::Condition as VehicleCondition;
use crate::domain::entities::models::vehicle::fuel::Fuel as VehicleFuel;
use crate::domain::entities::models::vehicle::manufacturer::Manufacturer as VehicleManufacturer;
use crate::domain::{
    entities::{item::Item, property::Property, vehicle::Vehicle},
    services::{error::DomainError, webscraping::marketplace::WebscrapingMarketplaceService},
};

const SEL_PHOTO_INPUT: &str = "input[type='file']";
const SEL_FACEBOOK_LOGGED_IN: &str = "div[aria-label='Facebook']";

const SEL_FACEBOOK_TRUST_DEVICE: &str = "div[data-testid='save-device-button'], \
                                          button[name='save_device'], \
                                          div[aria-label='Salvar dispositivo'], \
                                          .__7n5 button";

fn model_to_label(model: &crate::domain::entities::models::property::model::Model) -> &'static str {
    match model {
        crate::domain::entities::models::property::model::Model::Sale => "À venda",
        crate::domain::entities::models::property::model::Model::Rent => "Aluguel",
    }
}

fn category_to_label(
    category: &crate::domain::entities::models::property::category::Category,
) -> &'static str {
    match category {
        crate::domain::entities::models::property::category::Category::Apartment => "Apartamento",
        crate::domain::entities::models::property::category::Category::House => "Casa",
    }
}

pub fn vehicle_category_to_label(
    category: &crate::domain::entities::models::vehicle::category::Category,
) -> &'static str {
    match category {
        crate::domain::entities::models::vehicle::category::Category::CarOrPickup => "Carro/picape",
        crate::domain::entities::models::vehicle::category::Category::Motorcycle => "Motocicleta",
        crate::domain::entities::models::vehicle::category::Category::SportsVehicle => {
            "Veículos para esportes"
        }
        crate::domain::entities::models::vehicle::category::Category::Trailer => "Trailer",
        crate::domain::entities::models::vehicle::category::Category::UtilityTrailer => "Reboque",
        crate::domain::entities::models::vehicle::category::Category::Boat => "Barco",
        crate::domain::entities::models::vehicle::category::Category::CommercialOrIndustrial => {
            "Comercial/industrial"
        }
        crate::domain::entities::models::vehicle::category::Category::Other => "Outro",
    }
}

pub fn item_category_to_label(
    category: &crate::domain::entities::models::item::category::Category,
) -> &'static str {
    match category {
        crate::domain::entities::models::item::category::Category::Tools => "Ferramentas",
        crate::domain::entities::models::item::category::Category::Furniture => "Móveis",
        crate::domain::entities::models::item::category::Category::HouseholdItems => "Domésticos",
        crate::domain::entities::models::item::category::Category::Garden => "Jardim",
        crate::domain::entities::models::item::category::Category::Appliances => "Eletrodomésticos",
        crate::domain::entities::models::item::category::Category::VideoGames => "Videogames",
        crate::domain::entities::models::item::category::Category::BooksMoviesMusic => "Livros",
        crate::domain::entities::models::item::category::Category::BagsAndLuggage => {
            "Bolsas e malas"
        }
        crate::domain::entities::models::item::category::Category::WomensClothingAndShoes => {
            "femininas"
        }
        crate::domain::entities::models::item::category::Category::MensClothingAndShoes => {
            "masculinas"
        }
        crate::domain::entities::models::item::category::Category::JewelryAndAccessories => {
            "Joias e acessórios"
        }
        crate::domain::entities::models::item::category::Category::HealthAndBeauty => {
            "Saúde e beleza"
        }
        crate::domain::entities::models::item::category::Category::PetSupplies => "animais",
        crate::domain::entities::models::item::category::Category::BabiesAndKids => {
            "Bebês e crianças"
        }
        crate::domain::entities::models::item::category::Category::ToysAndGames => "Brinquedos",
        crate::domain::entities::models::item::category::Category::ElectronicsAndComputers => {
            "Eletrônicos"
        }
        crate::domain::entities::models::item::category::Category::CellPhones => "Celulares",
        crate::domain::entities::models::item::category::Category::Bicycles => "Bicicletas",
        crate::domain::entities::models::item::category::Category::ArtsAndCrafts => "Artesanato",
        crate::domain::entities::models::item::category::Category::AutoParts => "Peças",
        crate::domain::entities::models::item::category::Category::MusicalInstruments => {
            "Instrumentos musicais"
        }
        crate::domain::entities::models::item::category::Category::AntiquesAndCollectibles => {
            "Antiguidades"
        }
        crate::domain::entities::models::item::category::Category::GarageSale => "garagem",
        crate::domain::entities::models::item::category::Category::Miscellaneous => "Diversos",
    }
}

pub fn item_condition_to_label(
    condition: &crate::domain::entities::models::item::condition::Condition,
) -> &'static str {
    match condition {
        crate::domain::entities::models::item::condition::Condition::New => "Novo",
        crate::domain::entities::models::item::condition::Condition::UsedLikeNew => {
            "estado de novo"
        }
        crate::domain::entities::models::item::condition::Condition::UsedGood => "boas condições",
        crate::domain::entities::models::item::condition::Condition::UsedFair => {
            "condições razoáveis"
        }
    }
}

pub fn body_style_to_label(body_style: &VehicleBodyStyle) -> &'static str {
    match body_style {
        VehicleBodyStyle::Coupe => "Cupê",
        VehicleBodyStyle::Pickup => "Picape",
        VehicleBodyStyle::Sedan => "Sedã",
        VehicleBodyStyle::Hatchback => "Hatch",
        VehicleBodyStyle::Suv => "SUV",
        VehicleBodyStyle::Convertible => "Conversível",
        VehicleBodyStyle::StationWagon => "Station wagon",
        VehicleBodyStyle::Minivan => "Minivan",
        VehicleBodyStyle::CompactCar => "Carro compacto",
        VehicleBodyStyle::Other => "Outro",
    }
}

pub fn condition_to_label(condition: &VehicleCondition) -> &'static str {
    match condition {
        VehicleCondition::Excellent => "Excelente",
        VehicleCondition::VeryGood => "Muito bom",
        VehicleCondition::Good => "Bom",
        VehicleCondition::Fair => "Razoável",
        VehicleCondition::Poor => "Ruim",
    }
}

pub fn fuel_type_to_label(fuel: &VehicleFuel) -> &'static str {
    match fuel {
        VehicleFuel::Diesel => "Diesel",
        VehicleFuel::Electric => "Elétrico",
        VehicleFuel::Gasoline => "Gasolina",
        VehicleFuel::Flex => "Flex",
        VehicleFuel::Hybrid => "Híbrido",
        VehicleFuel::PlugInHybrid => "Híbrido plug-in",
        VehicleFuel::Other => "Outro",
    }
}

pub fn manufacturer_to_label(manufacturer: &VehicleManufacturer) -> &'static str {
    match manufacturer {
        VehicleManufacturer::AmGeneral => "AM General",
        VehicleManufacturer::Agrale => "Agrale",
        VehicleManufacturer::AlfaRomeo => "Alfa Romeo",
        VehicleManufacturer::AstonMartin => "Aston Martin",
        VehicleManufacturer::Audi => "Audi",
        VehicleManufacturer::Bmw => "BMW",
        VehicleManufacturer::Bentley => "Bentley",
        VehicleManufacturer::Cadillac => "Cadillac",
        VehicleManufacturer::Chery => "Chery",
        VehicleManufacturer::Chevrolet => "Chevrolet",
        VehicleManufacturer::Chrysler => "Chrysler",
        VehicleManufacturer::Citroen => "Citroën",
        VehicleManufacturer::CrossLander => "Cross Lander",
        VehicleManufacturer::Ds => "DS",
        VehicleManufacturer::Daewoo => "Daewoo",
        VehicleManufacturer::Daihatsu => "Daihatsu",
        VehicleManufacturer::Dodge => "Dodge",
        VehicleManufacturer::Effa => "Effa",
        VehicleManufacturer::Ferrari => "Ferrari",
        VehicleManufacturer::Fiat => "FIAT",
        VehicleManufacturer::Ford => "Ford",
        VehicleManufacturer::Gmc => "GMC",
        VehicleManufacturer::Geely => "Geely",
        VehicleManufacturer::Gurgel => "Gurgel",
        VehicleManufacturer::Hafei => "Hafei",
        VehicleManufacturer::Haima => "Haima",
        VehicleManufacturer::Honda => "Honda",
        VehicleManufacturer::Hummer => "Hummer",
        VehicleManufacturer::Hyundai => "Hyundai",
        VehicleManufacturer::Infiniti => "Infiniti",
        VehicleManufacturer::Iveco => "Iveco",
        VehicleManufacturer::Jac => "JAC",
        VehicleManufacturer::Jpx => "JPX",
        VehicleManufacturer::Jaguar => "Jaguar",
        VehicleManufacturer::Jeep => "Jeep",
        VehicleManufacturer::Kia => "Kia",
        VehicleManufacturer::Lada => "Lada",
        VehicleManufacturer::Lamborghini => "Lamborghini",
        VehicleManufacturer::LandRover => "Land Rover",
        VehicleManufacturer::Lexus => "Lexus",
        VehicleManufacturer::Lifan => "Lifan",
        VehicleManufacturer::Lotus => "Lotus",
        VehicleManufacturer::Mg => "MG",
        VehicleManufacturer::Mini => "MINI",
        VehicleManufacturer::Mahindra => "Mahindra",
        VehicleManufacturer::Maserati => "Maserati",
        VehicleManufacturer::Mazda => "Mazda",
        VehicleManufacturer::McLaren => "McLaren",
        VehicleManufacturer::MercedesBenz => "Mercedes-Benz",
        VehicleManufacturer::Mitsubishi => "Mitsubishi",
        VehicleManufacturer::Nissan => "Nissan",
        VehicleManufacturer::Puma => "PUMA",
        VehicleManufacturer::Peugeot => "Peugeot",
        VehicleManufacturer::Porsche => "Porsche",
        VehicleManufacturer::Renault => "Renault",
        VehicleManufacturer::RollsRoyce => "Rolls-Royce",
        VehicleManufacturer::Seat => "SEAT",
        VehicleManufacturer::Santana => "Santana",
        VehicleManufacturer::Shineray => "Shineray",
        VehicleManufacturer::Smart => "Smart",
        VehicleManufacturer::Ssangyong => "Ssangyong",
        VehicleManufacturer::Subaru => "Subaru",
        VehicleManufacturer::Suzuki => "Suzuki",
        VehicleManufacturer::Tac => "TAC",
        VehicleManufacturer::Toyota => "Toyota",
        VehicleManufacturer::Troller => "Troller",
        VehicleManufacturer::Volkswagen => "Volkswagen",
        VehicleManufacturer::Volvo => "Volvo",
        VehicleManufacturer::Voyage => "Voyage",
        VehicleManufacturer::ApcMotorCompany => "A.P.C Motor Company",
        VehicleManufacturer::Ajp => "AJP",
        VehicleManufacturer::Ajs => "AJS",
        VehicleManufacturer::Arc => "ARC",
        VehicleManufacturer::Arch => "Arch",
        VehicleManufacturer::Aspt => "ASPT",
        VehicleManufacturer::Adly => "Adly",
        VehicleManufacturer::Aermacchi => "Aermacchi",
        VehicleManufacturer::Alcyon => "Alcyon",
        VehicleManufacturer::Alta => "Alta",
        VehicleManufacturer::Ambassador => "Ambassador",
        VehicleManufacturer::AmericanIronhorse => "American IronHorse",
        VehicleManufacturer::AmericanLifan => "American Lifan",
        VehicleManufacturer::Apollo => "Apollo",
        VehicleManufacturer::Aprilia => "Aprilia",
        VehicleManufacturer::ArcticCat => "Arctic Cat",
        VehicleManufacturer::Ariane => "Ariane",
        VehicleManufacturer::Ariel => "Ariel",
        VehicleManufacturer::ArlenNess => "Arlen Ness",
        VehicleManufacturer::Armor => "Armor",
        VehicleManufacturer::Armstrong => "Armstrong",
        VehicleManufacturer::Artisan => "Artisan",
        VehicleManufacturer::Bmc => "BMC",
        VehicleManufacturer::Bms => "BMS",
        VehicleManufacturer::Bsa => "BSA",
        VehicleManufacturer::Bts => "BTS",
        VehicleManufacturer::Bajaj => "Bajaj",
        VehicleManufacturer::Baotian => "Baotian",
        VehicleManufacturer::Barossa => "Barossa",
        VehicleManufacturer::Bashan => "Bashan",
        VehicleManufacturer::Battistinis => "Battistinis",
        VehicleManufacturer::Beeline => "Beeline",
        VehicleManufacturer::Benelli => "Benelli",
        VehicleManufacturer::Beta => "Beta",
        VehicleManufacturer::Better => "Better",
        VehicleManufacturer::Bianchi => "Bianchi",
        VehicleManufacturer::BigBearChoppers => "Big Bear Choppers",
        VehicleManufacturer::BigDogMotorcycles => "Big Dog Motorcycles",
        VehicleManufacturer::Bimota => "Bimota",
        VehicleManufacturer::Bintelli => "Bintelli",
        VehicleManufacturer::Blata => "Blata",
        VehicleManufacturer::Boom => "Boom",
        VehicleManufacturer::Borile => "Borile",
        VehicleManufacturer::BossHoss => "Boss Hoss",
        VehicleManufacturer::Bourget => "Bourget",
        VehicleManufacturer::Bown => "Bown",
        VehicleManufacturer::Boxer => "Boxer",
        VehicleManufacturer::Brammo => "Brammo",
        VehicleManufacturer::Branson => "Branson",
        VehicleManufacturer::Bridgestone => "Bridgestone",
        VehicleManufacturer::Brixton => "Brixton",
        VehicleManufacturer::BroughSuperior => "Brough Superior",
        VehicleManufacturer::Buell => "Buell",
        VehicleManufacturer::Bullit => "Bullit",
        VehicleManufacturer::Bultaco => "Bultaco",
        VehicleManufacturer::Butler => "Butler",
        VehicleManufacturer::Ccm => "CCM",
        VehicleManufacturer::Cfmoto => "CFMOTO",
        VehicleManufacturer::ChRacing => "CH Racing",
        VehicleManufacturer::Cpi => "CPI",
        VehicleManufacturer::Cz => "CZ",
        VehicleManufacturer::Cagiva => "Cagiva",
        VehicleManufacturer::CaliforniaMotorcycleCompany => "California Motorcycle Company",
        VehicleManufacturer::CaliforniaScooterCompany => "California Scooter Company",
        VehicleManufacturer::Campagna => "Campagna",
        VehicleManufacturer::CanAm => "Can-Am",
        VehicleManufacturer::Cannondale => "Cannondale",
        VehicleManufacturer::Casal => "Casal",
        VehicleManufacturer::Cazador => "Cazador",
        VehicleManufacturer::Champ => "Champ",
        VehicleManufacturer::ChangJiang => "Chang Jiang",
        VehicleManufacturer::ChicagoScooterCo => "Chicago Scooter Co",
        VehicleManufacturer::Chunlan => "Chunlan",
        VehicleManufacturer::Cimatti => "Cimatti",
        VehicleManufacturer::Citycoco => "Citycoco",
        VehicleManufacturer::ClubCar => "Club Car",
        VehicleManufacturer::Cobra => "Cobra",
        VehicleManufacturer::Coleman => "Coleman",
        VehicleManufacturer::Condor => "Condor",
        VehicleManufacturer::Confederate => "Confederate",
        VehicleManufacturer::ConquestTrikes => "Conquest Trikes",
        VehicleManufacturer::Coolster => "Coolster",
        VehicleManufacturer::Cotton => "Cotton",
        VehicleManufacturer::Cougar => "Cougar",
        VehicleManufacturer::CoventryEagle => "Coventry Eagle",
        VehicleManufacturer::Cushman => "Cushman",
        VehicleManufacturer::Dkw => "DKW",
        VehicleManufacturer::Dot => "Dot",
        VehicleManufacturer::Daelim => "Daelim",
        VehicleManufacturer::Daix => "Daix",
        VehicleManufacturer::Dayang => "Dayang",
        VehicleManufacturer::DemonX => "Demon X",
        VehicleManufacturer::Derbi => "Derbi",
        VehicleManufacturer::DiBlasi => "Di Blasi",
        VehicleManufacturer::Diamo => "Diamo",
        VehicleManufacturer::Dichao => "Dichao",
        VehicleManufacturer::DirectBikes => "Direct Bikes",
        VehicleManufacturer::Dnepr => "Dnepr",
        VehicleManufacturer::DongFang => "Dong Fang",
        VehicleManufacturer::Douglas => "Douglas",
        VehicleManufacturer::Dresda => "Dresda",
        VehicleManufacturer::Ducati => "Ducati",
        VehicleManufacturer::Eton => "E-TON",
        VehicleManufacturer::EzGo => "E-Z-Go",
        VehicleManufacturer::Ebr => "EBR",
        VehicleManufacturer::EasyRider => "Easy-Rider",
        VehicleManufacturer::Eclipse => "Eclipse",
        VehicleManufacturer::ElectricMotion => "Electric Motion",
        VehicleManufacturer::Electricycle => "Electricycle",
        VehicleManufacturer::Energica => "Energica",
        VehicleManufacturer::Eped => "Eped",
        VehicleManufacturer::Erider => "Erider",
        VehicleManufacturer::ErikBuellRacing => "Erik Buell Racing",
        VehicleManufacturer::ExcelsiorHenderson => "Excelsior Henderson",
        VehicleManufacturer::Fantic => "Fantic",
        VehicleManufacturer::Feiying => "Feiying",
        VehicleManufacturer::Fenian => "Fenian",
        VehicleManufacturer::Fosti => "Fosti",
        VehicleManufacturer::FrancisBarnett => "Francis-Barnett",
        VehicleManufacturer::Garelli => "Garelli",
        VehicleManufacturer::GasGas => "Gas Gas",
        VehicleManufacturer::Genata => "Genata",
        VehicleManufacturer::Generic => "Generic",
        VehicleManufacturer::GenuineScooterCompany => "Genuine Scooter Company",
        VehicleManufacturer::GhezziBrian => "Ghezzi-Brian",
        VehicleManufacturer::Giantco => "Giantco",
        VehicleManufacturer::Gilera => "Gilera",
        VehicleManufacturer::Greeves => "Greeves",
        VehicleManufacturer::Grinnall => "Grinnall",
        VehicleManufacturer::Hanway => "Hanway",
        VehicleManufacturer::HaoNuo => "Hao Nuo",
        VehicleManufacturer::Haotian => "Haotian",
        VehicleManufacturer::HarleyDavidson => "Harley-Davidson",
        VehicleManufacturer::Harris => "Harris",
        VehicleManufacturer::Hartford => "Hartford",
        VehicleManufacturer::HellboundSteel => "Hellbound Steel",
        VehicleManufacturer::Herald => "Herald",
        VehicleManufacturer::Hesketh => "Hesketh",
        VehicleManufacturer::Himo => "Himo",
        VehicleManufacturer::Hisun => "Hisun",
        VehicleManufacturer::Honchin => "Honchin",
        VehicleManufacturer::Hongdu => "Hongdu",
        VehicleManufacturer::Honley => "Honley",
        VehicleManufacturer::Horex => "Horex",
        VehicleManufacturer::Huvo => "Huvo",
        VehicleManufacturer::Huatian => "Huatian",
        VehicleManufacturer::Huoniao => "Huoniao",
        VehicleManufacturer::Husaberg => "Husaberg",
        VehicleManufacturer::Husqvarna => "Husqvarna",
        VehicleManufacturer::Hyosung => "Hyosung",
        VehicleManufacturer::Isomoto => "Isomoto",
        VehicleManufacturer::Izh => "IZH",
        VehicleManufacturer::IceBear => "Ice Bear",
        VehicleManufacturer::Indian => "Indian",
        VehicleManufacturer::IronEagleMotorcycle => "Iron Eagle Motorcycle",
        VehicleManufacturer::Italika => "Italika",
        VehicleManufacturer::Italjet => "Italjet",
        VehicleManufacturer::Itom => "Itom",
        VehicleManufacturer::Jbwco => "Jbwco",
        VehicleManufacturer::Jcm => "JCM",
        VehicleManufacturer::James => "James",
        VehicleManufacturer::Jawa => "Jawa",
        VehicleManufacturer::Jialing => "Jialing",
        VehicleManufacturer::Jianshe => "Jianshe",
        VehicleManufacturer::Jincheng => "Jincheng",
        VehicleManufacturer::Jinfeng => "Jinfeng",
        VehicleManufacturer::Jinlun => "Jinlun",
        VehicleManufacturer::JohnDeere => "John Deere",
        VehicleManufacturer::JohnnyPag => "Johnny Pag",
        VehicleManufacturer::Jonway => "Jonway",
        VehicleManufacturer::Jotagas => "Jotagas",
        VehicleManufacturer::JuicyBike => "Juicy Bike",
        VehicleManufacturer::Kalex => "Kalex",
        VehicleManufacturer::Ktm => "KTM",
        VehicleManufacturer::Kangchao => "Kangchao",
        VehicleManufacturer::Kawasaki => "Kawasaki",
        VehicleManufacturer::Kayo => "Kayo",
        VehicleManufacturer::Keeway => "Keeway",
        VehicleManufacturer::KinroadRock => "Kinroad-Rock",
        VehicleManufacturer::Kymco => "Kymco",
        VehicleManufacturer::Lem => "Lem",
        VehicleManufacturer::Lambretta => "Lambretta",
        VehicleManufacturer::Lance => "Lance",
        VehicleManufacturer::Lanying => "Lanying",
        VehicleManufacturer::Laverda => "Laverda",
        VehicleManufacturer::Lehman => "Lehman",
        VehicleManufacturer::LehmanTrikes => "Lehman Trikes",
        VehicleManufacturer::Leike => "Leike",
        VehicleManufacturer::Levis => "Levis",
        VehicleManufacturer::Lexmoto => "Lexmoto",
        VehicleManufacturer::Lifan => "Lifan",
        VehicleManufacturer::Linhai => "Linhai",
        VehicleManufacturer::Lintex => "Lintex",
        VehicleManufacturer::Lml => "LML",
        VehicleManufacturer::Loncin => "Loncin",
        VehicleManufacturer::Longjia => "Longjia",
        VehicleManufacturer::Lyric => "Lyric",
        VehicleManufacturer::Mbk => "MBK",
        VehicleManufacturer::MvAgusta => "MV Agusta",
        VehicleManufacturer::Mz => "MZ",
        VehicleManufacturer::Magni => "Magni",
        VehicleManufacturer::Maico => "Maico",
        VehicleManufacturer::Malaguti => "Malaguti",
        VehicleManufacturer::Martin => "Martin",
        VehicleManufacturer::Masai => "Masai",
        VehicleManufacturer::Mash => "Mash",
        VehicleManufacturer::Matchless => "Matchless",
        VehicleManufacturer::Mavizen => "Mavizen",
        VehicleManufacturer::Maxus => "Maxus",
        VehicleManufacturer::Megelli => "Megelli",
        VehicleManufacturer::Mig => "Mig",
        VehicleManufacturer::Mikilon => "Mikilon",
        VehicleManufacturer::MiniMoto => "Mini Moto",
        VehicleManufacturer::Mobylette => "Mobylette",
        VehicleManufacturer::Modenas => "Modenas",
        VehicleManufacturer::Mondial => "Mondial",
        VehicleManufacturer::Montesa => "Montesa",
        VehicleManufacturer::Morini => "Morini",
        VehicleManufacturer::MotoGuzzi => "Moto Guzzi",
        VehicleManufacturer::MotoMartin => "Moto-Martin",
        VehicleManufacturer::MotoMorini => "Moto Morini",
        VehicleManufacturer::MotoParilla => "Moto-Parilla",
        VehicleManufacturer::Motobi => "Motobi",
        VehicleManufacturer::Motobecane => "Motobecane",
        VehicleManufacturer::MotorTrike => "Motor Trike",
        VehicleManufacturer::MotoHispania => "Moto-Hispania",
        VehicleManufacturer::Motorini => "Motorini",
        VehicleManufacturer::Motus => "Motus",
        VehicleManufacturer::Mutt => "Mutt",
        VehicleManufacturer::Metisse => "Metisse",
        VehicleManufacturer::Munch => "Munch",
        VehicleManufacturer::Nsu => "NSU",
        VehicleManufacturer::Nvt => "NVT",
        VehicleManufacturer::Neco => "Neco",
        VehicleManufacturer::NewHudson => "New Hudson",
        VehicleManufacturer::NewImperial => "New Imperial",
        VehicleManufacturer::NewMap => "New Map",
        VehicleManufacturer::Nippi => "Nippi",
        VehicleManufacturer::Nipponia => "Nipponia",
        VehicleManufacturer::Niu => "NIU",
        VehicleManufacturer::Norton => "Norton",
        VehicleManufacturer::Oset => "Oset",
        VehicleManufacturer::Omega => "Omega",
        VehicleManufacturer::Ossa => "Ossa",
        VehicleManufacturer::Pgo => "PGO",
        VehicleManufacturer::Por => "Por",
        VehicleManufacturer::Pannonia => "Pannonia",
        VehicleManufacturer::Panther => "Panther",
        VehicleManufacturer::ParamountCustomCycles => "Paramount Custom Cycles",
        VehicleManufacturer::Paton => "Paton",
        VehicleManufacturer::PeaceSports => "Peace Sports",
        VehicleManufacturer::Peirspeed => "Peirspeed",
        VehicleManufacturer::Pembleton => "Pembleton",
        VehicleManufacturer::Peripoli => "Peripoli",
        VehicleManufacturer::Petronas => "Petronas",
        VehicleManufacturer::Piaggio => "Piaggio",
        VehicleManufacturer::Pioneer => "Pioneer",
        VehicleManufacturer::PitsterPro => "Pitster Pro",
        VehicleManufacturer::Polaris => "Polaris",
        VehicleManufacturer::Polestar => "Polestar",
        VehicleManufacturer::Polini => "Polini",
        VehicleManufacturer::PrecisionCycleWorks => "Precision Cycle Works",
        VehicleManufacturer::Puch => "Puch",
        VehicleManufacturer::Pulse => "Pulse",
        VehicleManufacturer::Qingqi => "Qingqi",
        VehicleManufacturer::Quadro => "Quadro",
        VehicleManufacturer::Quantya => "Quantya",
        VehicleManufacturer::Raleigh => "Raleigh",
        VehicleManufacturer::RedHorse => "Red Horse",
        VehicleManufacturer::Regent => "Regent",
        VehicleManufacturer::Revtech => "Revtech",
        VehicleManufacturer::Rewaco => "Rewaco",
        VehicleManufacturer::Rhino => "Rhino",
        VehicleManufacturer::Rickman => "Rickman",
        VehicleManufacturer::Ridley => "Ridley",
        VehicleManufacturer::Rieju => "Rieju",
        VehicleManufacturer::Roadsmith => "Roadsmith",
        VehicleManufacturer::Roehr => "Roehr",
        VehicleManufacturer::Rokon => "Rokon",
        VehicleManufacturer::MotoRoma => "Moto-Roma",
        VehicleManufacturer::Romarsh => "Romarsh",
        VehicleManufacturer::Romeo => "Romeo",
        VehicleManufacturer::Rovigo => "Rovigo",
        VehicleManufacturer::Roxon => "Roxon",
        VehicleManufacturer::RoyalAlloy => "Royal Alloy",
        VehicleManufacturer::RoyalEnfield => "Royal Enfield",
        VehicleManufacturer::Rudge => "Rudge",
        VehicleManufacturer::Rumi => "Rumi",
        VehicleManufacturer::Sfm => "SFM",
        VehicleManufacturer::SsrMotorsports => "SSR Motorsports",
        VehicleManufacturer::Stacyc => "Stacyc",
        VehicleManufacturer::Swm => "SWM",
        VehicleManufacturer::Sym => "SYM",
        VehicleManufacturer::Sachs => "Sachs",
        VehicleManufacturer::Sanglas => "Sanglas",
        VehicleManufacturer::Sanya => "Sanya",
        VehicleManufacturer::Saxon => "Saxon",
        VehicleManufacturer::Schwinn => "Schwinn",
        VehicleManufacturer::Scomadi => "Scomadi",
        VehicleManufacturer::Scorpa => "Scorpa",
        VehicleManufacturer::Scott => "Scott",
        VehicleManufacturer::Secma => "Secma",
        VehicleManufacturer::Seeley => "Seeley",
        VehicleManufacturer::Segway => "Segway",
        VehicleManufacturer::Senke => "Senke",
        VehicleManufacturer::Sherco => "Sherco",
        VehicleManufacturer::Siamoto => "Siamoto",
        VehicleManufacturer::Silk => "Silk",
        VehicleManufacturer::Simson => "Simson",
        VehicleManufacturer::Sinnis => "Sinnis",
        VehicleManufacturer::Skygo => "Skygo",
        VehicleManufacturer::Skyjet => "Skyjet",
        VehicleManufacturer::Skyteam => "Skyteam",
        VehicleManufacturer::Slam => "Slam",
        VehicleManufacturer::Slingshot => "Slingshot",
        VehicleManufacturer::Smc => "Smc",
        VehicleManufacturer::Spirit => "Spirit",
        VehicleManufacturer::Spondon => "Spondon",
        VehicleManufacturer::StarMotorcycles => "Star Motorcycles",
        VehicleManufacturer::Starway => "Starway",
        VehicleManufacturer::Stomp => "Stomp",
        VehicleManufacturer::SuckerPunchSallys => "Sucker Punch Sallys",
        VehicleManufacturer::Sukida => "Sukida",
        VehicleManufacturer::Sumo => "Sumo",
        VehicleManufacturer::Sunbeam => "Sunbeam",
        VehicleManufacturer::SuperSoco => "Super Soco",
        VehicleManufacturer::Superbyke => "Superbyke",
        VehicleManufacturer::Swift => "Swift",
        VehicleManufacturer::Tgb => "TGB",
        VehicleManufacturer::TaizhouZhongneng => "Taizhou Zhongneng",
        VehicleManufacturer::Tamoretti => "Tamoretti",
        VehicleManufacturer::TaoMotor => "Tao Motor",
        VehicleManufacturer::TaoTao => "Tao Tao",
        VehicleManufacturer::Terrot => "Terrot",
        VehicleManufacturer::ThoroughbredMotorsports => "Thoroughbred Motorsports",
        VehicleManufacturer::Thumpstar => "Thumpstar",
        VehicleManufacturer::ThunderMountain => "Thunder Mountain",
        VehicleManufacturer::Titan => "Titan",
        VehicleManufacturer::Tmec => "Tmec",
        VehicleManufacturer::Tomos => "Tomos",
        VehicleManufacturer::Tor => "TOR",
        VehicleManufacturer::Track => "Track",
        VehicleManufacturer::Trailmaster => "Trailmaster",
        VehicleManufacturer::Triton => "Triton",
        VehicleManufacturer::Triumph => "Triumph",
        VehicleManufacturer::Ubco => "UBCO",
        VehicleManufacturer::Um => "UM",
        VehicleManufacturer::Ural => "Ural",
        VehicleManufacturer::Urban => "Urban",
        VehicleManufacturer::Vor => "Vor",
        VehicleManufacturer::Vanderhall => "Vanderhall",
        VehicleManufacturer::Vectrix => "Vectrix",
        VehicleManufacturer::Velocette => "Velocette",
        VehicleManufacturer::Vento => "Vento",
        VehicleManufacturer::Venture => "Venture",
        VehicleManufacturer::Vertemati => "Vertemati",
        VehicleManufacturer::Vertigo => "Vertigo",
        VehicleManufacturer::Vespa => "Vespa",
        VehicleManufacturer::Victoria => "Victoria",
        VehicleManufacturer::Victory => "Victory",
        VehicleManufacturer::Vincent => "Vincent",
        VehicleManufacturer::Vitacci => "Vitacci",
        VehicleManufacturer::Voxan => "Voxan",
        VehicleManufacturer::Vulcan => "Vulcan",
        VehicleManufacturer::Vyrus => "Vyrus",
        VehicleManufacturer::Velosolex => "Velosolex",
        VehicleManufacturer::WkBikes => "WK Bikes",
        VehicleManufacturer::Wangye => "Wangye",
        VehicleManufacturer::WildWest => "Wild West",
        VehicleManufacturer::WolfBrandScooters => "Wolf Brand Scooters",
        VehicleManufacturer::Wuyang => "Wuyang",
        VehicleManufacturer::Xgjao => "Xgjao",
        VehicleManufacturer::Xispa => "Xispa",
        VehicleManufacturer::XtremeMotorCo => "Xtreme Motor Co.",
        VehicleManufacturer::Ycf => "YCF",
        VehicleManufacturer::Yamaha => "Yamaha",
        VehicleManufacturer::Yamasaki => "Yamasaki",
        VehicleManufacturer::Yamoto => "Yamoto",
        VehicleManufacturer::Yiben => "Yiben",
        VehicleManufacturer::Yuan => "Yuan",
        VehicleManufacturer::Zenardi => "Zenardi",
        VehicleManufacturer::Zennco => "Zennco",
        VehicleManufacturer::Zero => "Zero",
        VehicleManufacturer::ZeroEngineering => "Zero Engineering",
        VehicleManufacturer::Zhejiang => "Zhejiang",
        VehicleManufacturer::Znen => "Znen",
        VehicleManufacturer::Zodiac => "Zodiac",
        VehicleManufacturer::Zongshen => "Zongshen",
        VehicleManufacturer::Zontes => "Zontes",
        VehicleManufacturer::Tm => "TM",
        VehicleManufacturer::Other => "Outro",
    }
}

fn profile_dir(client_id: &str) -> PathBuf {
    #[cfg(target_os = "windows")]
    let base = dirs::data_local_dir().unwrap_or_else(|| PathBuf::from("."));
    #[cfg(not(target_os = "windows"))]
    let base = dirs::data_dir().unwrap_or_else(|| PathBuf::from("."));

    let dir = base
        .join("fast-marketplace")
        .join("chrome-profiles")
        .join(client_id);
    std::fs::create_dir_all(&dir).ok();
    dir
}

#[async_trait]
pub trait PageExt {
    async fn wait_for_element(&self, selector: &str) -> Result<Element, DomainError>;
    async fn wait_for_xpath(&self, xpath: &str) -> Result<Element, DomainError>;
    async fn click_option(&self, text: &str) -> Result<(), DomainError>;
    async fn focus_and_type(&self, xpath: &str, value: &str) -> Result<(), DomainError>;
    async fn select_dropdown(&self, xpath: &str, option_text: &str) -> Result<(), DomainError>;
}

#[async_trait]
impl PageExt for Page {
    async fn wait_for_element(&self, selector: &str) -> Result<Element, DomainError> {
        for _ in 1..=20 {
            if let Ok(el) = self.find_element(selector).await {
                return Ok(el);
            }
            sleep(Duration::from_millis(500)).await;
        }
        Err(DomainError::AutomationError(format!(
            "Elemento não carregou na tela: {}",
            selector
        )))
    }

    async fn wait_for_xpath(&self, xpath: &str) -> Result<Element, DomainError> {
        for _ in 1..=20 {
            if let Ok(el) = self.find_xpath(xpath).await {
                return Ok(el);
            }
            sleep(Duration::from_millis(500)).await;
        }
        Err(DomainError::AutomationError(format!(
            "XPath não carregou na tela: {}",
            xpath
        )))
    }

    async fn click_option(&self, text: &str) -> Result<(), DomainError> {
        let xpath = format!("//*[@role='option'][contains(., '{}')]", text);
        let el = self.wait_for_xpath(&xpath).await?;
        el.click().await.map_err(|e| {
            DomainError::AutomationError(format!("Falha ao clicar na opção '{}': {}", text, e))
        })?;
        sleep(Duration::from_millis(300)).await;
        Ok(())
    }

    async fn focus_and_type(&self, xpath: &str, value: &str) -> Result<(), DomainError> {
        let el = self.wait_for_xpath(xpath).await?;
        el.click().await.map_err(|e| {
            DomainError::AutomationError(format!(
                "Falha ao clicar no input para focar ({}): {}",
                xpath, e
            ))
        })?;

        let js = format!(
            r#"(function() {{
                var el = document.evaluate({:?}, document, null,
                    XPathResult.FIRST_ORDERED_NODE_TYPE, null).singleNodeValue;
                if (!el) return false;
                el.focus();
                document.execCommand('insertText', false, {:?});
                return true;
            }})()"#,
            xpath, value
        );

        let ok = self
            .evaluate(js)
            .await
            .map_err(|e| {
                DomainError::AutomationError(format!(
                    "Falha ao injetar JS no input ({}): {}",
                    xpath, e
                ))
            })?
            .into_value::<bool>()
            .unwrap_or(false);

        if !ok {
            return Err(DomainError::AutomationError(format!(
                "O JS de digitação falhou no elemento: {}",
                xpath
            )));
        }

        Ok(())
    }

    async fn select_dropdown(&self, xpath: &str, option_text: &str) -> Result<(), DomainError> {
        let el = self.wait_for_xpath(xpath).await?;
        el.click().await.map_err(|e| {
            DomainError::AutomationError(format!("Falha ao clicar no dropdown ({}): {}", xpath, e))
        })?;
        sleep(Duration::from_secs(1)).await;
        self.click_option(option_text).await?;
        Ok(())
    }
}

pub struct FacebookMarketplaceService {
    browser: Mutex<Option<Browser>>,
    page: Mutex<Option<Page>>,
}

impl FacebookMarketplaceService {
    pub fn new() -> Self {
        Self {
            browser: Mutex::new(None),
            page: Mutex::new(None),
        }
    }

    async fn launch_browser(client_id: &str) -> Result<Browser, DomainError> {
        let (browser, mut handler) = Browser::launch(
            BrowserConfig::builder()
                .with_head()
                .user_data_dir(profile_dir(client_id))
                .arg("--start-maximized")
                .arg("--disable-infobars")
                .arg("--disable-notifications")
                .arg("--disable-blink-features=AutomationControlled")
                .arg("--no-sandbox")
                .arg("--disable-dev-shm-usage")
                .arg("--disable-web-security")
                .arg("--disable-features=IsolateOrigins,site-per-process")
                .arg("--allow-running-insecure-content")
                .arg("--disable-site-isolation-trials")
                .arg("--excludeSwitches=enable-automation")
                .arg("--useAutomationExtension=false")
                .arg("--user-agent=Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/136.0.0.0 Safari/537.36")
                .build()
                .map_err(|_| DomainError::NotFound)?,
        )
        .await
        .map_err(|_| DomainError::NotFound)?;

        tokio::task::spawn(async move {
            while let Some(h) = handler.next().await {
                if h.is_err() {
                    break;
                }
            }
        });

        Ok(browser)
    }

    pub async fn open(&self, url: &str, client_id: String) -> Result<(), DomainError> {
        let browser = Self::launch_browser(client_id.as_str()).await?;
        let page = browser
            .new_page(url)
            .await
            .map_err(|_| DomainError::NotFound)?;

        *self.browser.lock().await = Some(browser);
        *self.page.lock().await = Some(page);

        Ok(())
    }

    pub async fn close(&self) {
        if let Some(mut browser) = self.browser.lock().await.take() {
            let _ = browser.close().await;
        }
        *self.page.lock().await = None;
    }

    pub async fn login(&self, client_id: String) -> Result<(), DomainError> {
        let mut browser = Self::launch_browser(client_id.as_str()).await?;
        let page = browser
            .new_page("https://www.facebook.com/login")
            .await
            .map_err(|_| DomainError::NotFound)?;

        for _ in 0..240 {
            sleep(Duration::from_secs(2)).await;
            if let Ok(js_result) = page.evaluate("window.location.href").await {
                if let Ok(current_url) = js_result.into_value::<String>() {
                    let is_out_of_login = !current_url.contains("login")
                        && !current_url.contains("two_factor")
                        && !current_url.contains("two-factor")
                        && !current_url.contains("save-device")
                        && !current_url.contains("trust");

                    if is_out_of_login && page.find_element(SEL_FACEBOOK_LOGGED_IN).await.is_ok() {
                        let trust_prompt_visible =
                            page.find_element(SEL_FACEBOOK_TRUST_DEVICE).await.is_ok();

                        if trust_prompt_visible {
                            continue;
                        }

                        sleep(Duration::from_secs(4)).await;
                        let _ = browser.close().await;
                        return Ok(());
                    }
                }
            }
        }

        let _ = browser.close().await;
        Err(DomainError::NotFound)
    }

    async fn get_page<'a>(
        guard: &'a tokio::sync::MutexGuard<'a, Option<Page>>,
    ) -> Result<&'a Page, DomainError> {
        guard.as_ref().ok_or(DomainError::NotFound)
    }
}

impl Default for FacebookMarketplaceService {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl WebscrapingMarketplaceService for FacebookMarketplaceService {
    async fn add_property(&self, entity: Property, client_id: String) -> Result<(), DomainError> {
        const XPATH_MODEL_DROPDOWN: &str =
            "//label[@role='combobox'][contains(., 'venda ou locação')]";
        const XPATH_CATEGORY_DROPDOWN: &str =
            "//label[@role='combobox'][contains(., 'Tipo de imóvel')]";
        const XPATH_PARKING_DROPDOWN: &str =
            "//label[@role='combobox'][contains(., 'Vagas de estacionamento')]";
        const XPATH_BEDROOM_INPUT: &str =
            "//span[contains(., 'Número de quartos')]/following::input[1]";
        const XPATH_BATHROOM_INPUT: &str =
            "//span[contains(., 'Número de banheiros')]/following::input[1]";
        const XPATH_PRICE_INPUT: &str = "//span[contains(., 'Preço')]/following::input[1]";
        const XPATH_ADDRESS_INPUT: &str = "//input[@role='combobox'][@aria-autocomplete='list'][not(contains(@aria-label, 'Pesquisar'))]";
        const XPATH_DESCRIPTION_TEXTAREA: &str =
            "//span[contains(., 'Descrição do imóvel')]/following::textarea[1]";
        const XPATH_METER_INPUT: &str =
            "//span[contains(., 'Metros quadrados')]/following::input[1]";
        const XPATH_TAX_INPUT: &str = "//span[contains(., 'Imposto')]/following::input[1]";
        const XPATH_CONDOMINIUM_INPUT: &str =
            "//span[contains(., 'Condomínio')]/following::input[1]";

        let url = "https://www.facebook.com/marketplace/create/rental".to_string();

        self.open(&url, client_id).await?;

        let guard = self.page.lock().await;
        let page = Self::get_page(&guard).await?;

        page.evaluate(
            r#"
            Object.defineProperty(navigator, 'webdriver', {
                get: () => undefined,
                configurable: true
            });

            delete window.cdc_adoQpoasnfa76pfcZLmcfl_Array;
            delete window.cdc_adoQpoasnfa76pfcZLmcfl_Promise;
            delete window.cdc_adoQpoasnfa76pfcZLmcfl_Symbol;

            Object.defineProperty(navigator, 'plugins', {
                get: () => [1, 2, 3, 4, 5],
            });

            Object.defineProperty(navigator, 'languages', {
                get: () => ['pt-BR', 'pt', 'en-US', 'en'],
            });
        "#,
        )
        .await
        .map_err(|e| {
            DomainError::AutomationError(format!(
                "Falha ao simular uma pessoa real para o Chromium: {}",
                e
            ))
        })?;

        let el = page.wait_for_element(SEL_PHOTO_INPUT).await?;
        let image_paths: Vec<String> = entity.image().iter().map(|s| s.to_string()).collect();
        page.execute(SetFileInputFilesParams {
            files: image_paths,
            node_id: Some(el.node_id),
            backend_node_id: None,
            object_id: None,
        })
        .await
        .map_err(|e| {
            DomainError::AutomationError(format!("Falha ao enviar as fotos para o Chromium: {}", e))
        })?;

        sleep(Duration::from_secs(2)).await;

        page.select_dropdown(XPATH_MODEL_DROPDOWN, model_to_label(entity.model()))
            .await?;
        page.select_dropdown(
            XPATH_CATEGORY_DROPDOWN,
            category_to_label(entity.category()),
        )
        .await?;

        page.focus_and_type(XPATH_BEDROOM_INPUT, &entity.bedroom().to_string())
            .await?;
        page.focus_and_type(XPATH_BATHROOM_INPUT, &entity.bathroom().to_string())
            .await?;
        page.focus_and_type(XPATH_PRICE_INPUT, &entity.price().to_string())
            .await?;

        page.focus_and_type(XPATH_ADDRESS_INPUT, entity.address())
            .await?;
        sleep(Duration::from_millis(800)).await;

        if let Ok(el) = page.find_xpath("//*[@role='option'][1]").await {
            let _ = el.click().await;
        }

        page.focus_and_type(XPATH_DESCRIPTION_TEXTAREA, entity.description())
            .await?;
        page.focus_and_type(XPATH_METER_INPUT, &entity.meter().to_string())
            .await?;
        page.focus_and_type(XPATH_TAX_INPUT, &entity.tax().to_string())
            .await?;
        page.focus_and_type(XPATH_CONDOMINIUM_INPUT, &entity.condominium().to_string())
            .await?;

        page.select_dropdown(XPATH_PARKING_DROPDOWN, &entity.parking().to_string())
            .await?;

        let max_attempts = 240;
        for _ in 0..max_attempts {
            sleep(Duration::from_secs(2)).await;

            if let Ok(js_result) = page.evaluate("window.location.href").await {
                if let Ok(current_url) = js_result.into_value::<String>() {
                    if current_url.contains("marketplace/you/selling") {
                        break;
                    }
                }
            }
        }

        drop(guard);

        let _ = self.close().await;

        Ok(())
    }

    async fn add_vehicle(&self, entity: Vehicle, client_id: String) -> Result<(), DomainError> {
        const XPATH_TYPE_DROPDOWN: &str =
            "//label[@role='combobox'][contains(., 'Tipo de veículo')]";
        const XPATH_YEAR_DROPDOWN: &str = "//label[@role='combobox'][contains(., 'Ano')]";
        const XPATH_MAKE_DROPDOWN: &str = "//label[@role='combobox'][contains(., 'Fabricante')]";
        const XPATH_MAKE_INPUT: &str = "//span[contains(., 'Fabricante')]/following::input[1]";
        const XPATH_MODEL_INPUT: &str = "//span[contains(., 'Modelo')]/following::input[1]";
        const XPATH_MILEAGE_INPUT: &str =
            "//span[contains(., 'Quilometragem')]/following::input[1]";
        const XPATH_PRICE_INPUT: &str = "//span[contains(., 'Preço')]/following::input[1]";
        const XPATH_BODYSTYLE_DROPDOWN: &str =
            "//label[@role='combobox'][contains(., 'Estilo da carroceria')]";
        const XPATH_CONDITION_DROPDOWN: &str =
            "//label[@role='combobox'][contains(., 'Condição do veículo')]";
        const XPATH_FUEL_DROPDOWN: &str =
            "//label[@role='combobox'][contains(., 'Tipo de combustível')]";
        const XPATH_LOCATION_INPUT: &str = "//input[@role='combobox'][@aria-label='Localização']";
        const XPATH_DESCRIPTION_TEXTAREA: &str =
            "//span[contains(., 'Descrição')]/following::textarea[1]";
        const SEL_PHOTO_INPUT: &str = "input[type='file'][accept*='image']";

        let url = "https://www.facebook.com/marketplace/create/vehicle".to_string();

        self.open(&url, client_id).await?;

        let guard = self.page.lock().await;
        let page = Self::get_page(&guard).await?;

        page.evaluate(
            r#"
            Object.defineProperty(navigator, 'webdriver', {
                get: () => undefined,
                configurable: true
            });

            delete window.cdc_adoQpoasnfa76pfcZLmcfl_Array;
            delete window.cdc_adoQpoasnfa76pfcZLmcfl_Promise;
            delete window.cdc_adoQpoasnfa76pfcZLmcfl_Symbol;

            Object.defineProperty(navigator, 'plugins', {
                get: () => [1, 2, 3, 4, 5],
            });

            Object.defineProperty(navigator, 'languages', {
                get: () => ['pt-BR', 'pt', 'en-US', 'en'],
            });
        "#,
        )
        .await
        .map_err(|e| {
            DomainError::AutomationError(format!(
                "Falha ao simular uma pessoa real para o Chromium: {}",
                e
            ))
        })?;

        let el = page.wait_for_element(SEL_PHOTO_INPUT).await?;
        let image_paths: Vec<String> = entity.image().iter().map(|s| s.to_string()).collect();

        page.execute(SetFileInputFilesParams {
            files: image_paths,
            node_id: Some(el.node_id),
            backend_node_id: None,
            object_id: None,
        })
        .await
        .map_err(|e| {
            DomainError::AutomationError(format!("Falha ao enviar as fotos para o Chromium: {}", e))
        })?;

        sleep(Duration::from_secs(2)).await;

        page.select_dropdown(
            XPATH_TYPE_DROPDOWN,
            vehicle_category_to_label(entity.category()),
        )
        .await?;

        sleep(Duration::from_secs(2)).await;

        page.select_dropdown(XPATH_YEAR_DROPDOWN, &entity.year().to_string())
            .await?;

        match entity.category() {
            VehicleCategory::CarOrPickup
            | VehicleCategory::Motorcycle
            | VehicleCategory::CommercialOrIndustrial => {
                page.select_dropdown(
                    XPATH_MAKE_DROPDOWN,
                    manufacturer_to_label(entity.manufacturer()),
                )
                .await?;
            }
            _ => {
                page.focus_and_type(
                    XPATH_MAKE_INPUT,
                    manufacturer_to_label(entity.manufacturer()),
                )
                .await?;
            }
        }

        page.focus_and_type(XPATH_MODEL_INPUT, &entity.model())
            .await?;

        if page.find_xpath(XPATH_MILEAGE_INPUT).await.is_ok() {
            let _ = page
                .focus_and_type(XPATH_MILEAGE_INPUT, &entity.mileage().to_string())
                .await;
        }

        if page.find_xpath(XPATH_BODYSTYLE_DROPDOWN).await.is_ok() {
            let _ = page
                .select_dropdown(
                    XPATH_BODYSTYLE_DROPDOWN,
                    body_style_to_label(entity.bodystyle()),
                )
                .await;
        }

        if page.find_xpath(XPATH_CONDITION_DROPDOWN).await.is_ok() {
            let _ = page
                .select_dropdown(
                    XPATH_CONDITION_DROPDOWN,
                    condition_to_label(entity.condition()),
                )
                .await;
        }

        if page.find_xpath(XPATH_FUEL_DROPDOWN).await.is_ok() {
            let _ = page
                .select_dropdown(XPATH_FUEL_DROPDOWN, fuel_type_to_label(entity.fuel()))
                .await;
        }

        page.focus_and_type(XPATH_PRICE_INPUT, &entity.price().to_string())
            .await?;

        page.focus_and_type(XPATH_LOCATION_INPUT, &entity.address())
            .await?;
        sleep(Duration::from_millis(800)).await;

        if let Ok(el) = page.find_xpath("//*[@role='option'][1]").await {
            let _ = el.click().await;
        }

        if page.find_xpath(XPATH_DESCRIPTION_TEXTAREA).await.is_ok() {
            let _ = page
                .focus_and_type(XPATH_DESCRIPTION_TEXTAREA, &entity.description())
                .await;
        }

        let max_attempts = 240;
        for _ in 0..max_attempts {
            sleep(Duration::from_secs(2)).await;

            if let Ok(js_result) = page.evaluate("window.location.href").await {
                if let Ok(current_url) = js_result.into_value::<String>() {
                    if current_url.contains("marketplace/you/selling") {
                        break;
                    }
                }
            }
        }

        drop(guard);

        let _ = self.close().await;

        Ok(())
    }

    async fn add_item(&self, _entity: Item, _client_id: String) -> Result<(), DomainError> {
        todo!()
    }
}
