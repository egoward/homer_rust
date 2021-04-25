use rumqttc::{MqttOptions, Client, QoS, Event, Packet};
use std::time::Duration;
use std::thread;
use std::str;

pub fn main_mqtt() {

    let mut mqttoptions = MqttOptions::new("rumqtt-sync", "house", 1883);
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
        //client.publish("hello/rumqtt", QoS::AtLeastOnce, false, vec![i; i as usize]).unwrap();
        client.publish("hello/rumqtt", QoS::AtLeastOnce, false, data).unwrap();
        //println!("i = {}", i);
        thread::sleep(Duration::from_millis(100));
    };

    poller.join().unwrap();
}