use serde::{Serialize, Deserialize};
use rusoto_cloudwatch::*;
use rusoto_cloudwatch as cw;
use rusoto_core::Region;
pub use super::core::*;
use async_trait::async_trait;

#[derive(Deserialize,Serialize)]
pub struct DestinationCloudwatchConfig {
    pub namespace: String,
    pub region: String,
    pub profile: String
}

impl DestinationCloudwatchConfig {
    pub fn example_config()->DestinationCloudwatchConfig {
        return DestinationCloudwatchConfig {
            namespace:"TestCloudwatchNamespace".to_string(),
            profile:"edonica".to_string(),
            region:"eu-west-2".to_string()
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

        let mut profile = rusoto_core::credential::ProfileProvider::new().unwrap();
        let request_dispatcher = rusoto_core::request::HttpClient::new().expect("failed to create request dispatcher");
        profile.set_profile(self.profile.clone());

        let region = Region::EuWest2; //Region::from_str(self.region).unwrap();

        let client = cw::CloudWatchClient::new_with(request_dispatcher, profile, region);

        return Box::new( DestinationCloudwatch{
            config : self,
            name: n,
            client 
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

        println!("{} Sending {} metrics",self.name(), metrics.len());

        let datums: Vec<MetricDatum> = metrics.into_iter().map( |metric| MetricDatum {
            metric_name: metric.property.clone(),
            storage_resolution: Some(60),
            value: metric.value.parse().ok(),
            dimensions: Some(vec![Dimension {
                name:"Object".to_string(),
                value: metric.object.clone()
            }]),
            counts:None,
            statistic_values:None,
            timestamp:None,
            unit:None,
            values:None
        }).collect();

        let input = PutMetricDataInput{
            metric_data: datums,
            namespace: self.config.namespace.clone(),
        };

        self.client.put_metric_data(input).await.expect("Send metric to CloudWatch");
        println!("{} sent",self.name());
    }

    fn shutdown(&mut self) -> Option<std::thread::JoinHandle<()>> {
        return None;
    }
}

