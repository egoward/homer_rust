mod ble;
mod mqtt;
mod homer_relay;

use homer_relay::core::*;
use homer_relay::mqtt::*;

use structopt::StructOpt;


/// Search for a pattern in a file and display the lines that contain it.
#[derive(Debug,StructOpt)]
#[structopt(about = "The stupid message relay")]
enum Command {
    #[structopt(about = "Test some BLE functionality")]
    TestBLE {
        device : String
    },
    #[structopt(about = "Test some MQTT functionality")]
    TestMQTT {
    },
    #[structopt(about = "Run the thing")]
    Run {
    }
}

#[derive(Debug,StructOpt)]
struct CommandLine {
    #[structopt(name = "config", default_value = "config.toml", long = "config")]
    config_file: String,
    //#[structopt(name = "v", long = "verbose")]
    #[structopt(name = "v", long = "verbose")]
    verbose: bool,
    #[structopt(subcommand)]  // Note that we mark a field as a subcommand
    cmd: Command
}



fn main() {

    let args = CommandLine::from_args();

    if args.verbose {
        println!("{:?}",args);

        println!("Using config from {}", args.config_file);
    }



    match &args.cmd {
        Command::TestMQTT {} => {
            mqtt::main_mqtt();
        }
        Command::TestBLE { device: _ } => {
            ble::main_ble();
        }
        Command::Run {} => {

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
        }

    }
}