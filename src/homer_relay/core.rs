use serde::{Serialize, Deserialize};
use async_trait::async_trait;

pub struct Metric {
    pub object: String,
    pub property: String,
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

#[typetag::serde(tag = "type")]
pub trait SourceConfig {
    fn name(&self) -> String;
    fn init(self : Box<Self>) -> Box<dyn Source>;
}

#[async_trait]
pub trait Source {
    fn name(&self) -> &String;
    async fn poll(&mut self) -> Vec<Metric>;
    fn shutdown(&mut self) -> Option<std::thread::JoinHandle<()>> {
        return Option::None;
    }
}


#[derive(Deserialize,Serialize)]
pub struct Config {
    pub destinations : Vec<Box<dyn DestinationConfig>>,
    pub sources : Vec<Box<dyn SourceConfig>>
}

pub struct Manager {
    pub destinations : Vec<Box<dyn Destination>>, 
    pub sources : Vec<Box<dyn Source>> 
}

impl Manager {
    pub fn create( config : Config ) -> Manager {
        println!("manager - creating from configuration");
        let mut manager = Manager {
            destinations: vec![],
            sources: vec![]
        };
        for destination_conf in config.destinations {
            println!("manager - creating {}", destination_conf.name());
            let destination = destination_conf.init();
            println!("manager - created {}", destination.name());
            manager.destinations.push(destination)
        }
        for source_conf in config.sources {
            println!("manager - creating {}", source_conf.name());
            let source = source_conf.init();
            println!("manager - created {}", source.name());
            manager.sources.push(source)
        }
        return manager;
    }

    pub async fn send_metrics(&mut self, metrics: &Vec<Metric>) -> () {
        for destination in &mut self.destinations {
            println!("manager - sending test metric to {}", destination.name());
            destination.report( &metrics ).await;
        }  
    }

    pub async fn test(&mut self) {
        println!("manager - sending test metric to all destinations");
        let metrics = vec![Metric {
            object: String::from("TestSensor"),
            property: String::from("Temperature"),
            value: String::from("1.23"),
        }];
        self.send_metrics(&metrics).await;
    }       

    pub async fn run(&mut self) {
        let mut all_metrics : Vec<Metric>=vec![];

        for source in &mut self.sources {
            println!("manager - shutting down {}", source.name());
            let metrics = source.poll().await;
            for metric in metrics {
                all_metrics.push(metric);
            }
        }
        self.send_metrics(&all_metrics).await;
    }

    pub fn shutdown(&mut self) {
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
        for source in &mut self.sources {
            println!("manager - shutting down {}", source.name());
            match source.shutdown() {
                Some(join_handle) => {
                    println!("manager - {} is busy, waiting", source.name() );
                    join_handle.join().unwrap();
                    println!("manager - {} is done", source.name() );
                }
                None => {
                }
            }
        }
    }
}

