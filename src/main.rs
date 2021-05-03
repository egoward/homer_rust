mod ble;
mod homer_relay;

use structopt::StructOpt;
use std::thread;
use std::time::Duration;

// use homer_relay::core::*;
use homer_relay::mqtt::*;
use homer_relay::cloudwatch::*;

/// Search for a pattern in a file and display the lines that contain it.
#[derive(Debug,StructOpt)]
#[structopt(about = "The stupid message relay")]
enum Command {
    #[structopt(about = "Test sending to destinations")]
    TestSend {
    },
    #[structopt(about = "Test Bluetooth Low Energy")]
    TestBLE {
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
    #[structopt(name = "v", long = "verbose")]
    verbose: bool,
    #[structopt(subcommand)]
    cmd: Command
}

fn write_example_config() {
    let config = Config {
        destinations : vec! {
            Box::new( DestinationLogConfig {} ),
            Box::new( DestinationMQTTConfig::example_config()),
            Box::new( DestinationCloudwatchConfig::example_config()),
        }
    };


    let test_output : String = toml::to_string(&config).unwrap();
    let filename = "config.example.toml";
    println!("Writing example configuration to {}", filename);
    std::fs::write(filename, test_output).unwrap();
}


#[tokio::main]
async fn main() {
    let args = CommandLine::from_args();

    if args.verbose {
        println!("Verbose mode!");
        println!("Arguments : {:?}",args);
        println!("Using config from {}", args.config_file);
    }

    let config_content = match std::fs::read_to_string(&args.config_file) {
        Ok(file) => file,
        Err(error) => {
            println!("Error opening file \"{}\"", &args.config_file);
            panic!("Error : {:?}",error)
        },
    };

     let config: Config = match toml::from_str(&config_content) {
        Ok(m) => m,
        Err(error) => {
            println!("Error reading configuration file \"{}\"", &args.config_file);
            write_example_config();            
            panic!("Error : {:?}",error)
        },
     };

     if args.verbose {
        // We can't Debug our metric manager now
        // println!("Configuration : {:?}",metricManager);
     }

    ctrlc::set_handler(move || {
        println!("received Ctrl+C!");
    }).unwrap();

    match &args.cmd {
        Command::TestSend {} => {
            let mut manager = Manager::create( config );
            manager.test().await;
            manager.shutdown();
            println!("Waiting for a second in case there's stuff in the background");
            thread::sleep(Duration::from_millis(1000));
            println!("Done");

        }
        Command::TestBLE {} => {
            ble::main_ble();
        }
        Command::WriteExampleConfig {} => {
            write_example_config();
        }

        Command::Run {} => {
            let manager = Manager::create( config );
            manager.run();
        }

    }
}