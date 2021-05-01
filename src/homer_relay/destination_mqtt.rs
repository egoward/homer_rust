pub use super::homer_core::*;

pub struct MetricDestinationMQTT {}


impl MetricDestination for MetricDestinationMQTT {
    fn name(&self) -> String {
        return String::from("MetricDestinationMQTT");
    }

    fn report(&self, metrics: &Vec<Metric>) {
        for metric in metrics {
            println!("MQTT : Metric {} has value {}", metric.name, metric.value);
        }
    }
}

