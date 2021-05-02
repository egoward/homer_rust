mod ble;
mod homer_relay;

use structopt::StructOpt;

// use homer_relay::core::*;
use homer_relay::mqtt::*;

/// Search for a pattern in a file and display the lines that contain it.
#[derive(Debug,StructOpt)]
#[structopt(about = "The stupid message relay")]
enum Command {
    #[structopt(about = "Test configured destinations")]
    TestDestination {
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

fn writeExampleConfig() {
    let manager = MetricManager {
        sources : vec! {
            Box::new( MetricSourceTest {} )
        },
        destinations : vec! {
            Box::new( DestinationLogConfig {} ),
            Box::new( DestinationMQTTConfig {server:"localhost".to_string(),port:123, agent_name:"MessageRelayAgent".to_string()} ),
        }
    };


    let test_output : String = toml::to_string(&manager).unwrap();
    let filename = "config.example.toml";
    println!("Writing example configuration to {}", filename);
    std::fs::write(filename, test_output).unwrap();
}


fn main() {
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

     let manager: MetricManager = match toml::from_str(&config_content) {
        Ok(m) => m,
        Err(error) => {
            println!("Error reading configuration file \"{}\"", &args.config_file);
            writeExampleConfig();            
            panic!("Error : {:?}",error)
        },
     };

     if args.verbose {
        // We can't Debug our metric manager now
        // println!("Configuration : {:?}",metricManager);
     }

    match &args.cmd {
        Command::TestDestination {} => {
            manager.init();
            for destination in manager.destinations {
                println!("Testing {}", destination.name());
                let x = destination.init();
                x.test();
            }
        }
        Command::TestBLE {} => {
            ble::main_ble();
        }
        Command::WriteExampleConfig {} => {
            writeExampleConfig();
        }

        Command::Run {} => {
            manager.init();
            manager.run();
        }

    }
}