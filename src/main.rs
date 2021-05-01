mod ble;
mod homer_relay;

use structopt::StructOpt;
use serde::{Serialize, Deserialize};

use homer_relay::core::*;
use homer_relay::mqtt::*;

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
    },
    #[structopt(about = "Write an example configuration file")]
    WriteExampleConfig {
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

#[derive(Deserialize,Serialize,Debug)]
struct ConfigMain {
    title: String,
    mqtt: ConfigMQTT,
}


fn main() {

    let args = CommandLine::from_args();

    if args.verbose {
        println!("Verbose mode!");
        println!("Arguments : {:?}",args);
        println!("Using config from {}", args.config_file);
    }

    //let file_result = ;
    let config_content = match std::fs::read_to_string(&args.config_file) {
        Ok(file) => file,
        Err(error) => {
            println!("Error opening file \"{}\"", &args.config_file);
            panic!("Error : {:?}",error)
        },
    };

     //let config_content = fileResult.unwrap();
     let config: ConfigMain = match toml::from_str(&config_content) {
        Ok(config) => config,
        Err(error) => {
            println!("Error reading configuration file \"{}\"", &args.config_file);
            panic!("Error : {:?}",error)
        },
     };

     if args.verbose {
        println!("Configuration : {:?}",config);
     }

    match &args.cmd {
        Command::TestMQTT {} => {
            config.mqtt.test();
            //mqtt::main_mqtt();
        }
        Command::TestBLE { device: _ } => {
            ble::main_ble();
        }
        Command::WriteExampleConfig {} => {
            let example_config = ConfigMain {
                title : "foo".to_string(),
                mqtt : ConfigMQTT::example()
            };
            let test_output : String = toml::to_string(&example_config).unwrap();
            let filename = "config.example.toml";
            println!("Writing example configuration to {}", filename);
            std::fs::write(filename, test_output).unwrap();
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