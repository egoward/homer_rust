use btle1;

fn main() {
    println!("This is main");
    //btle1::test_some_local_function();

    let manager = btle1::MetricManager {
        sources : vec! {
            Box::new( btle1::MetricSourceTest {} )
        },
        destinations : vec! {
            Box::new( btle1::MetricDestinationLog {} )
        }
    };

    manager.run();

    btle1::main_mqtt();

}