mod ble;
mod mqtt;
mod homer_relay;

use homer_relay::core::*;
use homer_relay::mqtt::*;

fn main() {
    println!("This is main");
    //btle1::test_some_local_function();

    let manager = MetricManager {
        sources : vec! {
            Box::new( MetricSourceTest {} )
        },
        destinations : vec! {
            Box::new( MetricDestinationLog {} ),
            Box::new( MetricDestinationMQTT {} )
        }
    };

    manager.run();

    mqtt::main_mqtt();

    //ble::main_ble();
    
}