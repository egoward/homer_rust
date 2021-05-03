use serde::{Serialize, Deserialize};
use rusoto_cloudwatch::*;
use rusoto_cloudwatch as cw;
use rusoto_core::Region;
pub use super::core::*;

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

impl Destination for DestinationCloudwatch {
    fn name(&self) -> &String { 
        return &self.name;
    }


    fn report(&mut self, metrics: &Vec<super::core::Metric>) {
        for metric in metrics {
            println!("{} Sending metric {} with value {}", self.name(), metric.name, metric.value);

            fn bar() -> impl std::future::Future<> {
                // This `async` block results in a type that implements
                // `Future<Output = u8>`.
                async move {
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

                    let client2 = cw::CloudWatchClient::new(Region::default());
                    let result = client2.put_metric_data(input).await;
                    println!("Done send");
                }
            }


        }
    }

    fn shutdown(&mut self) -> Option<std::thread::JoinHandle<()>> {
        return None;
    }
}

