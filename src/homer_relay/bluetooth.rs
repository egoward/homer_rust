pub use super::core::*;

use serde::{Serialize, Deserialize};
use async_trait::async_trait;

#[derive(Deserialize,Serialize)]
pub struct SourceBLEConfig {
    id : String,
}

pub struct SourceBLE {
    config : Box<SourceBLEConfig>,
    name: String,
}

impl SourceBLEConfig {
    pub fn example_config()->SourceBLEConfig {
        return SourceBLEConfig {
            id: "123".to_string(),
        }
    }
}

#[typetag::serde(name = "ble")]
impl SourceConfig for SourceBLEConfig {
    fn name(&self) -> String {
        return format!("bluetooth {}", self.id);
    }
    fn init(self : Box<Self>) -> Box<dyn Source> {
        return Box::new( SourceBLE{
            name: self.name(),
            config: self
        } )
    }
}


#[async_trait]
impl Source for SourceBLE {
    fn name(&self) -> &String { 
        return &self.name;
    }
    async fn poll(&mut self) -> Vec<Metric> {
        println!("{} - returning value", self.name());
        return vec![/*Metric {
            object: self.config.object.clone(),
            property: self.config.property.clone(),
            value: format!("{}", self.config.value)
        }*/]
        
    }
}

