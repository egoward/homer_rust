use rumqttc::{MqttOptions, Client, QoS, Event, Packet, Outgoing};
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
    pub publish_channel: String,

}

impl DestinationMQTTConfig {
    pub fn example_config()->DestinationMQTTConfig {
        return DestinationMQTTConfig {server:"localhost".to_string(),port:1883, agent_name:"MessageRelayAgent".to_string(), publish_channel: "/MetricRelay/".to_string()}
    }
}

#[typetag::serde(name = "mqtt")]
impl DestinationConfig for DestinationMQTTConfig {
    fn name(&self) -> String {
        return String::from("MetricDestinationMQTT");
    }

    fn init(self : Box<Self> ) -> Box<dyn Destination> {

        let mut mqttoptions = MqttOptions::new( self.agent_name.clone(), self.server.clone(), self.port);
        mqttoptions.set_keep_alive(5);

        let (client, mut connection) = Client::new(mqttoptions, 10);


        let name = format!("mqtt {}:{}", self.server, self.port);
        let n = name.clone();

        //The connection belongs to the thread...
        let poller = thread::spawn( move || {
            println!("{} : Poller thread started", n);
            for (_, notification) in connection.iter().enumerate() {
                match notification {
                    Ok(msg)  => {
                        match &msg {
                            Event::Incoming(incoming_msg) => {
                                match &incoming_msg {
                                     Packet::Publish(packet) => {
                                        let msg_string = str::from_utf8(&packet.payload).unwrap();
                                        println!("{} : Packet = {}", n, msg_string);
                                    }
                                    Packet::PubAck(_publish_message) => {
                                        println!("{} Published",n);
                                    }
                                    _ => {
                                        println!("{} In:{:?}", n, incoming_msg);
                                    }
        
                                }
                            }
                            Event::Outgoing(outgoing_msg) => {
                                match &outgoing_msg {
                                    Outgoing::Disconnect => {
                                        println!("{} Got disconnect", n);
                                        break;
                                    }
                                    _ => {
                                        println!("{} Out:{:?}", n, outgoing_msg);
                                    }
                                }
                            }
        
                        }
                    }
                    Err(e) => {
                        println!("{} Error:{:?}", n, e);
                    }
                }
            }
            println!("{} Poll thread exiting", n );
        });    

        return Box::new( DestinationMQTT{
            config : self,
            name,
            client,
            poller: Some(poller)
        } )

    }
}


pub struct DestinationMQTT {
    name: String,
    config : Box<DestinationMQTTConfig>,
    client : Client,
    poller : Option<std::thread::JoinHandle<()>>,

}

impl Destination for DestinationMQTT {
    fn name(&self) -> &String { 
        return &self.name;
    }

    fn report(&mut self, metrics: &Vec<Metric>) {
        for metric in metrics {
            
            //println!("MQTT : Metric {} has value {}", metric.name, metric.value);

            //println!("Sending stuff");
            let msg_content = format!("{}",metric.value);
            let channel = format!("{}{}",&self.config.publish_channel, &metric.name);
            let data = msg_content.as_bytes();
            println!("{} publish {} : {}", self.name(), channel, msg_content);
            self.client.publish(channel, QoS::AtLeastOnce, false, data).unwrap();
            thread::sleep(Duration::from_millis(100));

        }
    }

    fn shutdown(&mut self) -> Option<std::thread::JoinHandle<()>> {
        println!("{} - disconnecting", self.name());
        self.client.disconnect().unwrap();
        return self.poller.take();
    }
}

