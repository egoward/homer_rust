use rumqttc::{MqttOptions, Client,  QoS, Event, Packet};
use std::time::Duration;
use std::thread;
use std::str;
use serde::{Serialize, Deserialize};

pub use super::core::*;

#[derive(Deserialize,Serialize)]
pub struct DestinationMQTTConfig {
    pub server: String,
    #[serde(default)]
    pub port: u16,
    #[serde(default)]
    pub agent_name: String,

}

impl DestinationMQTTConfig {
    pub fn example_config()->DestinationMQTTConfig {
        return DestinationMQTTConfig {server:"localhost".to_string(),port:123, agent_name:"MessageRelayAgent".to_string()}        
    }
}

#[typetag::serde(name = "mqtt")]
impl DestinationConfig for DestinationMQTTConfig {
    fn name(&self) -> String {
        return String::from("MetricDestinationMQTT");
    }

    fn init(self : Box<Self> ) -> Box<dyn Destination> {
        return Box::new( DestinationMQTT{} )
    }
}


pub struct DestinationMQTT {
    //client : Client,
    //connection : Connection,
}

impl Destination for DestinationMQTT {
    fn test(&self) {
        let mut mqttoptions = MqttOptions::new("mqtt-agent", "house", 1883);
        mqttoptions.set_keep_alive(5);
        
        println!("Doing subscribe");
        let (mut client, mut connection) = Client::new(mqttoptions, 10);
        client.subscribe("hello/rumqtt", QoS::AtMostOnce).unwrap();
    
        let poller = thread::spawn( move || {
            println!("Polling for stuff");
            // Iterate to poll the eventloop for connection progress
            for (i, notification) in connection.iter().enumerate() {
                println!("Notification = {} {:?}", i, notification);
                let msg = notification.unwrap();
                if let Event::Incoming(incoming_msg) = msg {
                    if let Packet::Publish(packet) = incoming_msg {
                        let msg_string = str::from_utf8(&packet.payload).unwrap();
                        println!("packet = {}", msg_string);
                        if msg_string == "Hello 9" {
                            break;
                        }
                    }
                }
            }
            println!("Done???");
    
        });
    
        
        println!("Sending stuff");
        for i in 0..10 {
            let msg_content = format!("Hello {}",i);
            let data = msg_content.as_bytes();
            client.publish("hello/rumqtt", QoS::AtLeastOnce, false, data).unwrap();
            thread::sleep(Duration::from_millis(100));
        };
    
        poller.join().unwrap();
    }
        
    fn report(&self, metrics: &Vec<Metric>) {
        for metric in metrics {
            println!("MQTT : Metric {} has value {}", metric.name, metric.value);
        }
    }
}

