use uuid::Uuid;

#[allow(unused_imports)]
use rand::{thread_rng, Rng};
// use simple_logger::SimpleLogger;
#[allow(dead_code)]
#[allow(unused_imports)]
use std::thread;
use std::time::Duration;
use std::sync::Arc;

#[allow(unused_imports)]
use btleplug::api::{Central, Characteristic, Peripheral};
#[allow(unused_imports)]
#[cfg(target_os = "linux")]
use btleplug::bluez::{adapter::Adapter, manager::Manager};
#[allow(unused_imports)]
#[cfg(target_os = "macos")]
use btleplug::corebluetooth::{adapter::Adapter, manager::Manager};
#[allow(unused_imports)]
#[cfg(target_os = "windows")]
use btleplug::winrtble::{adapter::Adapter, manager::Manager};
use btleplug::api::{bleuuid::BleUuid, CentralEvent};

#[cfg(target_os = "linux")]
fn print_adapter_info(adapter: &Adapter) {
    println!(
        "connected adapter {:?} is powered: {:?}",
        adapter.name(),
        adapter.is_powered()
    );
}


#[cfg(any(target_os = "windows", target_os = "macos"))]
fn print_adapter_info(_adapter: &Adapter) {
    println!("adapter info can't be printed on Windows 10 or mac");
}

pub use super::core::*;

use serde::{Serialize, Deserialize};
use async_trait::async_trait;

#[derive(Deserialize,Serialize)]
pub struct SourceBLEConfig {
    id : String,
}

pub struct SourceBLE {
    config : Box<SourceBLEConfig>,
    name: String,
}

impl SourceBLEConfig {
    pub fn example_config()->SourceBLEConfig {
        return SourceBLEConfig {
            id: "123".to_string(),
        }
    }
}

#[typetag::serde(name = "ble")]
impl SourceConfig for SourceBLEConfig {
    fn name(&self) -> String {
        return format!("bluetooth {}", self.id);
    }
    fn init(self : Box<Self>) -> Box<dyn Source> {
        return Box::new( SourceBLE{
            name: self.name(),
            config: self
        } )
    }
}


#[async_trait]
impl Source for SourceBLE {
    fn name(&self) -> &String { 
        return &self.name;
    }
    async fn poll(&mut self) -> Vec<Metric> {
        println!("{} - returning value", self.name());
        return vec![/*Metric {
            object: self.config.object.clone(),
            property: self.config.property.clone(),
            value: format!("{}", self.config.value)
        }*/]
        
    }
}

pub struct BleManager {
    manager : Manager,
    adapter : Adapter,
    bluetooth_db : Arc<BluetoothDB>,
    poller : Option<std::thread::JoinHandle<()>>
}

impl BleManager {

    pub fn create() -> BleManager {
        println!("Testing bluetooth");

        let bluetooth_db = Arc::new(BluetoothDB::create());

        let manager = Manager::new().unwrap();
        let adapter_list : Vec<Adapter> = manager.adapters().unwrap();
        
        println!("Adapters : {}", adapter_list.len() );

        let adapter = adapter_list.into_iter().nth(0).unwrap();

//        let x = adapter.connect().unwrap();

        print_adapter_info(&adapter);

        let event_receiver = adapter.event_receiver().unwrap();


        let dbRef = Arc::clone(&bluetooth_db);
        let poller = thread::spawn( move || {
            println!("Bluetooth Poller started");
            while let Ok(event) = event_receiver.recv() {
                match event {
                    CentralEvent::DeviceDiscovered(bd_addr) => {
                        println!("DeviceDiscovered: {:?}", bd_addr);
                    }
                    CentralEvent::DeviceConnected(bd_addr) => {
                        println!("DeviceConnected: {:?}", bd_addr);
                    }
                    CentralEvent::DeviceDisconnected(bd_addr) => {
                        println!("DeviceDisconnected: {:?}", bd_addr);
                    }
                    CentralEvent::ManufacturerDataAdvertisement {
                        address,
                        manufacturer_id,
                        data,
                    } => {
                        println!(
                            "ManufacturerDataAdvertisement: {:?}, {} {}, {:x?}",
                            address, manufacturer_id, dbRef.getCompany( manufacturer_id ), data
                        );

                    }
                    CentralEvent::ServiceDataAdvertisement {
                        address,
                        service,
                        data,
                    } => {
                        println!(
                            "ServiceDataAdvertisement: {:?}, {}, {:x?}",
                            address,
                            service.to_string(),
                            //service.to_short_string(),
                            data
                        );
                    }
                    CentralEvent::ServicesAdvertisement { address, services } => {
                        let services: Vec<String> =
                            services.into_iter().map(|s| s.to_short_string()).collect();
                        println!("ServicesAdvertisement: {:?}, {:?}", address, services);
                    }                    
                    e => {
                        println!("Event recevied {:?}",e);
                    }
                }
            }            
            println!("Loop finished");
        });

        //let adapter = adapter_list.remove(0);
        return BleManager {
            manager,
            adapter,
            bluetooth_db : bluetooth_db,
            poller: Some(poller)

        };
    }

    fn shutdown(&mut self) -> Option<std::thread::JoinHandle<()>> {
        println!("Terminating bluetooth");
        //self.adapter.listener..drop();
        println!("Bluetooth disconnecting");
        return self.poller.take();
    }



    pub fn scan(&mut self, duration : Duration , ctrl_channel : crossbeam_channel::Receiver<()>) {
        println!("Doing scan for {:?} ...", duration);
        self.adapter.start_scan().unwrap();

        let ticks = crossbeam_channel::tick(duration);

        loop {
            crossbeam_channel::select! {
                recv(ticks) -> _ => {
                    println!("Finished waiting");
                    break;
                }
                recv(ctrl_channel) -> _ => {
                    println!("Aborting due to Ctrl-C!");
                    break;
                }
            }
        }
        println!("Done");
    }

    pub fn list(&mut self) {
        for peripheral in self.adapter.peripherals().iter() {
            //let p : &dyn Peripheral = peripheral;
            println!(
                "peripheral : {:?} {:?} is connected: {:?}",
                peripheral.address(),
                peripheral.properties().local_name,
                peripheral.is_connected()
            );

            match peripheral.discover_characteristics() {
                Err(error) => {
                    println!("  Error : {:?}",error)
                }
                Ok(characteristics) => { /*{Vec<Characteristic>} */
                    for ch in characteristics {
                        println!("  {:?}",ch)
                    }
                }
            }
        }

        println!("Done");
    }

    pub fn connect(&mut self, _id: String) {
        self.adapter.peripherals().iter().find( |x| {
            println!("peripheral : {:?}",x.address());
            return false;

        });
    }

}

#[derive(Deserialize)]
struct CompanyJSON {
    code: u16,
    name: String,
}   

#[derive(Deserialize)]
struct BluetoothMetadata {
    name: String,
    identifier: String,
    uuid: String,
    source: String
}

pub struct BluetoothDB {
    mapCompany : std::collections::HashMap<u16, String>,
    mapCharacteristic : std::collections::HashMap<Uuid, BluetoothMetadata>,
    mapService : std::collections::HashMap<Uuid, BluetoothMetadata>,
    mapDescriptor : std::collections::HashMap<Uuid, BluetoothMetadata>,

}

 


impl BluetoothDB {


    fn readNameCodeFile(filename : &str)-> std::collections::HashMap<u16, String> {
        println!("Parsing {}", filename);
        let file = std::fs::File::open(filename).unwrap();
        let json : Vec<CompanyJSON> = serde_json::from_reader(file).unwrap();
        return json.into_iter().map( |x| (x.code, x.name)).collect();
    }

    fn parseUUID( string : &str) -> Uuid {
        let baseUUID : Uuid = Uuid::parse_str("00000000-0000-1000-8000-00805F9B34FB").unwrap();
        //let (a,b,c,d) = baseUUID.as_fields();
        
        
        if string.len() == 4 {
            let stuff = u32::from_str_radix(string, 16);
            //let ret = baseUUID;
            return baseUUID;
        } else if string.len() == 36  {
            return Uuid::parse_str( string ).unwrap();
        } else {
            panic!("Unexpected length of uuid {} ({})" , string, string.len())
        }
    }

    fn readDescriptorFile(filename : &str) -> std::collections::HashMap<Uuid, BluetoothMetadata> {
        println!("Parsing {}", filename);
        let file = std::fs::File::open(filename).unwrap();
        let json : Vec<BluetoothMetadata> = serde_json::from_reader(file).unwrap();
        let ret : std::collections::HashMap<Uuid, BluetoothMetadata> = json.into_iter().map( |x| 
            (BluetoothDB::parseUUID(&x.uuid), x ) 
        ).collect();

        for (k,v) in ret.iter() {
            println!(" {} => {}", k, v.name);

        }

        return ret;
    }

    pub fn create() -> BluetoothDB {
        return BluetoothDB {
            mapCompany : BluetoothDB::readNameCodeFile("data/bluetooth-numbers-database/v1/company_ids.json"),
            mapCharacteristic : BluetoothDB::readDescriptorFile("data/bluetooth-numbers-database/v1/characteristic_uuids.json"),
            mapService : BluetoothDB::readDescriptorFile("data/bluetooth-numbers-database/v1/service_uuids.json"),
            mapDescriptor : BluetoothDB::readDescriptorFile("data/bluetooth-numbers-database/v1/descriptor_uuids.json"),
        }
    }

    fn getCompany(&self, id : u16) -> String {
        match self.mapCompany.get(&id) {
            Some(v) => {return v.clone();}
            None => {return format!("Unknown({})",id)}
        }
    }

}
