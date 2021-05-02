use serde::{Serialize, Deserialize};

pub struct Metric {
    pub name: String,
    pub value: String,
}

#[typetag::serde(tag = "type")]
pub trait DestinationConfig {
    fn name(&self) -> String;
    fn init(&self) -> Box<dyn Destination>;
}

pub trait Destination {
    fn test(&self) -> () {
        println!("No tests applicable");
    }
    fn report(&self, metrics: &Vec<Metric>) -> ();
}


#[derive(Deserialize,Serialize)]
pub struct DestinationLogConfig {}

pub struct DestinationLog {}

#[typetag::serde(name = "log")]
impl DestinationConfig for DestinationLogConfig {
    fn name(&self) -> String {
        return String::from("MetricDestinationLog");
    }
    fn init(&self) -> Box<dyn Destination> {
        return Box::new( DestinationLog{} )
    }
}

impl Destination for DestinationLog {
    fn report(&self, metrics: &Vec<Metric>) {
        for metric in metrics {
            println!("Metric {} has value {}", metric.name, metric.value);
        }
    }
}


#[typetag::serde(tag = "type")]
pub trait MetricSource {
    fn name(&self) -> String;
    fn poll(&self) -> Vec<Metric>;
}

#[derive(Deserialize,Serialize)]
pub struct MetricSourceTest {}

#[typetag::serde]
impl MetricSource for MetricSourceTest {
    fn name(&self) -> String {
        return String::from("MetricSourceTest");
    }
    fn poll(&self) -> Vec<Metric> {
        return vec![Metric {
            name: String::from("TestMetric"),
            value: String::from("1.0"),
        }];
    }
}

#[derive(Deserialize,Serialize)]
pub struct MetricManager {
    pub destinations : Vec<Box<dyn DestinationConfig>>,
    //pub destinationImps : Vec<Box<dyn Destination>>,
    pub sources : Vec<Box<dyn MetricSource>>,
}

impl MetricManager {
    pub fn init(&self) {
        for destination in &self.destinations {
            println!("Initializing {}", destination.name());
            destination.init();
        }        
    }
    pub fn run(&self) {
        println!("Running metric manager");
        for source in &self.sources {
            println!("Checking {}", source.name());
            let metrics = source.poll();
            for destination in &self.destinations {
                println!("Sending to {}", destination.name());
                destination.init().report( &metrics );
            }
        }
    }
}