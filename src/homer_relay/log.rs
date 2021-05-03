pub use super::core::*;

use serde::{Serialize, Deserialize};
use async_trait::async_trait;

#[derive(Deserialize,Serialize)]
pub struct DestinationLogConfig {}

pub struct DestinationLog {
    name : String
}

#[typetag::serde(name = "log")]
impl DestinationConfig for DestinationLogConfig {
    fn name(&self) -> String {
        return String::from("MetricDestinationLog");
    }
    fn init(self : Box<Self>) -> Box<dyn Destination> {
        return Box::new( DestinationLog{
            name: "log".to_string()
        } )
    }
}

#[async_trait]
impl Destination for DestinationLog {
    fn name(&self) -> &String { 
        return &self.name;
    }
    async fn report(&mut self, metrics: &Vec<Metric>) {
        for metric in metrics {
            println!("{} - object {} has a {} of {}", self.name(), metric.object, metric.property, metric.value);
        }
    }
}

