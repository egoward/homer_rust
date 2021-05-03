use serde::{Serialize, Deserialize};
use rusoto_cloudwatch::*;
use rusoto_cloudwatch as cw;
use rusoto_core::Region;
pub use super::core::*;
use async_trait::async_trait;

#[derive(Deserialize,Serialize)]
pub struct DestinationCloudwatchConfig {
    pub namespace: String,
}

impl DestinationCloudwatchConfig {
    pub fn example_config()->DestinationCloudwatchConfig {
        return DestinationCloudwatchConfig {
            namespace:"TestCloudwatchNamespace".to_string()
        }
    }
}

#[typetag::serde(name = "cloudwatch")]
impl DestinationConfig for DestinationCloudwatchConfig {
    fn name(&self) -> String {
        return String::from("cloudwatch");
    }

    fn init(self : Box<Self> ) -> Box<dyn Destination> {
        let n = self.name().clone();
        return Box::new( DestinationCloudwatch{
            config : self,
            name: n,
            client: cw::CloudWatchClient::new(Region::default())
        } )

    }
}


pub struct DestinationCloudwatch {
    name: String,
    config : Box<DestinationCloudwatchConfig>,
    client: cw::CloudWatchClient,
}

#[async_trait]
impl Destination for DestinationCloudwatch {
    fn name(&self) -> &String { 
        return &self.name;
    }


    async fn report(&mut self, metrics: &Vec<super::core::Metric>) {
        for metric in metrics {
            println!("{} Sending metric {} with value {}", self.name(), metric.name, metric.value);

            let input = PutMetricDataInput{
                metric_data: vec![MetricDatum{
                    metric_name: "Temperature".to_string(),
                    storage_resolution: Some(60),
                    value: Some(1.0),
                    dimensions: Some(vec![Dimension {
                        name:"Object".to_string(),
                        value:"TestObject".to_string()
                    }]),
                    counts:None,
                    statistic_values:None,
                    timestamp:None,
                    unit:None,
                    values:None
                }],
                namespace: "TestNamespace".to_string(), //self.config.namespace.clone()
            };

            //let client2 = cw::CloudWatchClient::new(Region::default());
            println!("{} Sending metric 1",self.name());
            let w = self.client.put_metric_data(input);
            println!("{} Sending metric Awaiting",self.name());
            let value = w.await;
            println!("{} Waiting metric - value {:?}",self.name(),value);
            
            println!("{} Waiting for send to complete",self.name());
            //futures::executor::block_on(future); // `future` is run and "hello, world!" is printed
            println!("{} done",self.name());


        }
    }

    fn shutdown(&mut self) -> Option<std::thread::JoinHandle<()>> {
        return None;
    }
}

