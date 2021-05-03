pub use super::core::*;

use serde::{Serialize, Deserialize};
use async_trait::async_trait;

#[derive(Deserialize,Serialize)]
pub struct SourceConstantConfig {
    object : String,
    property : String,
    value : f64
}

pub struct SourceConstant {
    config : Box<SourceConstantConfig>,
    name: String,
}

impl SourceConstantConfig {
    pub fn example_config()->SourceConstantConfig {
        return SourceConstantConfig {
            object: "TestObject".to_string(),
            property: "Temperature".to_string(),
            value: 2.34
        }
    }
}

#[typetag::serde(name = "constant")]
impl SourceConfig for SourceConstantConfig {
    fn name(&self) -> String {
        return format!("constant Object {} : {}{}", self.object, self.property, self.value);
    }
    fn init(self : Box<Self>) -> Box<dyn Source> {
        return Box::new( SourceConstant{
            name: self.name(),
            config: self
        } )
    }
}


#[async_trait]
impl Source for SourceConstant {
    fn name(&self) -> &String { 
        return &self.name;
    }
    async fn poll(&mut self) -> Vec<Metric> {
        println!("{} - returning value", self.name());
        return vec![Metric {
            object: self.config.object.clone(),
            property: self.config.property.clone(),
            value: format!("{}", self.config.value)
        }]
        
    }
}

