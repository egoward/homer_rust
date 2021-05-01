mod ble;
mod mqtt;
mod homer_relay;

use homer_relay::homer_core::*;

fn main() {
    println!("This is main");
    //btle1::test_some_local_function();

    let manager = MetricManager {
        sources : vec! {
            Box::new( MetricSourceTest {} )
        },
        destinations : vec! {
            Box::new( MetricDestinationLog {} ),
            //Box::new( btle1::MetricDestinationMQTT {} )
        }
    };

    manager.run();

    mqtt::main_mqtt();

    //ble::main_ble();
    
}