use serde::{Serialize, Deserialize};

pub struct Metric {
    pub name: String,
    pub value: String,
}

#[typetag::serde(tag = "type")]
pub trait DestinationConfig {
    fn name(&self) -> String;
    fn init(self : Box<Self>) -> Box<dyn Destination>;
}

pub trait Destination {
    fn name(&self) -> String;
    fn report(&mut self, metrics: &Vec<Metric>) -> ();

    fn test(&mut self) -> () {
        println!("{} : No tests applicable", self.name());
    }
    fn shutdown(&mut self) -> () {
        println!("{} : No shutdown logic", self.name());
    }
}


#[derive(Deserialize,Serialize)]
pub struct DestinationLogConfig {}

pub struct DestinationLog {}

#[typetag::serde(name = "log")]
impl DestinationConfig for DestinationLogConfig {
    fn name(&self) -> String {
        return String::from("MetricDestinationLog");
    }
    fn init(self : Box<Self>) -> Box<dyn Destination> {
        return Box::new( DestinationLog{} )
    }
}

impl Destination for DestinationLog {
    fn name(&self) -> String { 
        return "DestinationLog".to_string() 
    }
    fn report(&mut self, metrics: &Vec<Metric>) {
        for metric in metrics {
            println!("Metric {} has value {}", metric.name, metric.value);
        }
    }
}



#[derive(Deserialize,Serialize)]
pub struct Config {
    pub destinations : Vec<Box<dyn DestinationConfig>>,
}

pub struct Manager {
    pub destinations : Vec<Box<dyn Destination>> 
}

impl Manager {
    pub fn create( config : Config ) -> Manager {
        let mut manager = Manager {
            destinations: vec![]
        };
        for destination in config.destinations {
            manager.destinations.push(destination.init())

        }
        return manager;
    }

    pub fn test(&mut self) {
        println!("Running metric manager");
        //for source in &self.sources {
            //println!("Checking {}", source.name());
            //let metrics = source.poll();
        let metrics = vec![Metric {
            name: String::from("TestMetric"),
            value: String::from("1.0"),
        }];
        for destination in &mut self.destinations {
            println!("Sending to {}", destination.name());
            destination.report( &metrics );
        }        
    }       

    pub fn run(&self) {

    }

    pub fn shutdown(&mut self) {
        println!("Shutting down");
        for destination in &mut self.destinations {
            destination.shutdown();
        }
    }
}

