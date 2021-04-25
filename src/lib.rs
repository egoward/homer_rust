mod ble;
mod mqtt;
pub use ble::main_ble;
pub use mqtt::main_mqtt;

pub fn test_some_local_function() {
    println!("This is test()");
}

pub struct Metric {
    name : String,
    value : String,
}

pub trait Destination {
    fn Report( Vec<Metric> metrics )->()

}

