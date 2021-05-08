mod ble;
mod homer_relay;

use structopt::StructOpt;
use std::thread;
use std::time::Duration;
use log;

// use homer_relay::core::*;
use homer_relay::log::*;
use homer_relay::mqtt::*;
use homer_relay::cloudwatch::*;
use homer_relay::constant::*;
use homer_relay::bluetooth::*;

/// Search for a pattern in a file and display the lines that contain it.
#[derive(Debug,StructOpt)]
#[structopt(about = "The stupid message relay")]
enum Command {
    #[structopt(about = "Test sending to destinations")]
    TestSend {
    },
    #[structopt(about = "Test Bluetooth Low Energy")]
    BLETest {
    },
    #[structopt(about = "Scan for bluetooth devices")]
    BLEScan {
        #[structopt(about = "Duration of scan")]
        duration : u64
    },
    #[structopt(about = "Connect to a device")]
    BLEConnect {
        #[structopt(name = "id", long = "id")]
        id: String,
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
        },
        sources : vec! {
            Box::new( SourceConstantConfig::example_config()),
            Box::new( SourceBLEConfig::example_config())
        }
    };


    let test_output : String = toml::to_string(&config).unwrap();
    let filename = "config.example.toml";
    println!("Writing example configuration to {}", filename);
    std::fs::write(filename, test_output).unwrap();
}

fn ctrl_channel() -> Result<crossbeam_channel::Receiver<()>, ctrlc::Error> {
    let (sender, receiver) = crossbeam_channel::bounded(100);
    ctrlc::set_handler(move || {
        println!("Received Ctrl-C");
        let _ = sender.send(());
    })?;

    Ok(receiver)
}

#[tokio::main]
async fn main() {
    println!("Parsing  arguments");
    let args = CommandLine::from_args();
    println!("Got this : {:?}",args);

    let log_level = match args.verbose {
        true => log::LevelFilter::Trace,
        false => log::LevelFilter::Info
    };

    simple_logger::SimpleLogger::new().with_level(log_level).init().unwrap();

    if args.verbose {
        log::trace!("Verbose mode!");
        log::trace!("Arguments : {:?}",args);
    }
    log::info!("Using config from {}", args.config_file);

    let config_content = match std::fs::read_to_string(&args.config_file) {
        Ok(file) => file,
        Err(error) => {
            log::error!("Error opening file \"{}\"", &args.config_file);
            panic!("Error : {:?}",error)
        },
    };

     let config: Config = match toml::from_str(&config_content) {
        Ok(m) => m,
        Err(error) => {
            log::error!("Error reading configuration file \"{}\"", &args.config_file);
            write_example_config();            
            panic!("Error : {:?}",error)
        },
     };

     if args.verbose {
        // We can't Debug our metric manager now
        // println!("Configuration : {:?}",metricManager);
     }


     let ctrl_c_events = ctrl_channel().unwrap();

    match &args.cmd {
        Command::TestSend {} => {
            let mut manager = Manager::create( config );
            manager.test().await;
            manager.shutdown();
            println!("Waiting for a second in case there's stuff in the background");
            thread::sleep(Duration::from_millis(1000));
            println!("Done");

        }
        Command::BLETest {} => {
            ble::main_ble();
        }
        Command::WriteExampleConfig {} => {
            write_example_config();
        }
        Command::BLEScan{duration} => {
            let mut x = BleManager::create();
            x.scan(Duration::from_secs(*duration), ctrl_c_events);
            x.list();
            x.shutdown();
        },
        Command::BLEConnect {id} => {
            let mut x = BleManager::create();
            x.scan(Duration::from_secs(10),ctrl_c_events);
            x.connect(id.clone());
            x.shutdown();
        }        
        Command::Run {} => {
            let mut manager = Manager::create( config );
            manager.run().await;
        }

    }
}