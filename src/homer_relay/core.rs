use serde::{Serialize, Deserialize};
use async_trait::async_trait;

pub struct Metric {
    pub name: String,
    pub value: String,
}

#[typetag::serde(tag = "type")]
pub trait DestinationConfig {
    fn name(&self) -> String;
    fn init(self : Box<Self>) -> Box<dyn Destination>;
}

#[async_trait]
pub trait Destination {
    fn name(&self) -> &String;
    async fn report(&mut self, metrics: &Vec<Metric>) -> ();

    fn test(&mut self) -> () {
        println!("{} : No tests applicable", self.name());
    }
    fn shutdown(&mut self) -> Option<std::thread::JoinHandle<()>> {
        return Option::None;
    }
}


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
            println!("{} - metric {} has value {}", self.name(), metric.name, metric.value);
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
        println!("manager - creating from configuration");
        let mut manager = Manager {
            destinations: vec![]
        };
        for destination_conf in config.destinations {
            println!("manager - creating {}", destination_conf.name());
            let destination = destination_conf.init();
            println!("manager - created {}", destination.name());
            manager.destinations.push(destination)

        }
        return manager;
    }

    pub async fn test(&mut self) {
        println!("manager - sending test metric to all destinations");
        let metrics = vec![Metric {
            name: String::from("TestMetric"),
            value: String::from("1.0"),
        }];
        for destination in &mut self.destinations {
            println!("manager - sending test metric to {}", destination.name());
            destination.report( &metrics ).await;
        }        
    }       

    pub fn run(&self) {

    }

    pub fn shutdown(mut self) {
        println!("manager - shutting down");
        for destination in &mut self.destinations {
            println!("manager - shutting down {}", destination.name());
            match destination.shutdown() {
                Some(join_handle) => {
                    println!("manager - {} is busy, waiting", destination.name() );
                    join_handle.join().unwrap();
                    println!("manager - {} is done", destination.name() );
                }
                None => {
                }
            }
        }
    }
}

