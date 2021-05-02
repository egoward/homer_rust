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

        let mut mqttoptions = MqttOptions::new("mqtt-agent", "house", 1883);
        mqttoptions.set_keep_alive(5);

        let (client, mut connection) = Client::new(mqttoptions, 10);

        //The connection belongs to the thread...
        let poller = thread::spawn( move || {
            println!("Polling for stuff");
            // Iterate to poll the eventloop for connection progress
            for (i, notification) in connection.iter().enumerate() {
                match notification {
                    Ok(msg)  => {
                        println!("Notification = {} [{:?}]", i, msg);
                        match &msg {
                            Event::Incoming(incoming_msg) => {
                                match &incoming_msg {
                                     Packet::Publish(packet) => {
                                        let msg_string = str::from_utf8(&packet.payload).unwrap();
                                        println!("packet = {}", msg_string);
                
                                        //println!("Publish message");
                                    }
                                    Packet::PubAck(_publish_message) => {
                                        println!("Publish ack");
                                    }
                                    Packet::PingResp => {
                                        println!("PingResp");
                                    }
                                    _ => {
                                        println!("Something else");
                                    }
        
                                }
                                //println!("Incoming message");
                                
                            }
                            Event::Outgoing(outgoing_msg) => {
                                match &outgoing_msg {
                                    Outgoing::Disconnect => {
                                        println!("Publish message");
                                    }
                                    _ => {
                                        println!("Some outgoing message");
                                    }
                                }
                                //println!("Outgoing message");
                            }
        
                        }
                    }
                    Err(e) => {
                        println!("Error {:?}",e)

                    }
                }

                /*
                if let Event::Incoming(incoming_msg) = msg {
                    if let Packet::Publish(packet) = incoming_msg {
                        let msg_string = str::from_utf8(&packet.payload).unwrap();
                        println!("packet = {}", msg_string);
                        if msg_string == "Hello 9" {
                            break;
                        }
                    } 
                } else {
                    println!("Not an incoming message");
                }
                */
            }
            println!("Done???");

        });    

        return Box::new( DestinationMQTT{
            config : self,
            client : client,
            poller: poller
        } )

    }
}


pub struct DestinationMQTT {
    config : Box<DestinationMQTTConfig>,
    client : Client,
    #[allow(dead_code)]
    poller : std::thread::JoinHandle<()>,

}

impl Drop for DestinationMQTT {
    fn drop(&mut self) {
        println!("Dropping DestinationMQTT");
        self.client.disconnect().unwrap();
        //self.poller.join().unwrap();
        //drop self.poller;
        println!("Dropped DestinationMQTT");

    }

}

impl Destination for DestinationMQTT {
    fn name(&self) -> String { 
        return format!("DestinationMQTT {}:{}", self.config.server, self.config.port);
    }

    fn test(&mut self) {
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
        
    fn report(&mut self, metrics: &Vec<Metric>) {
        for metric in metrics {
            println!("MQTT : Metric {} has value {}", metric.name, metric.value);

            println!("Sending stuff");
            let msg_content = format!("{}",metric.value);
            let data = msg_content.as_bytes();
            let channel = format!("{}{}",&self.config.publish_channel, &metric.name);
            self.client.publish(channel, QoS::AtLeastOnce, false, data).unwrap();
            thread::sleep(Duration::from_millis(100));

        }
    }

    fn shutdown(&mut self) -> () {
        println!("Disconnecting");
        self.client.disconnect().unwrap();
        //self.poller.join().unwrap();
        return;
    }
}

