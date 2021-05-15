pub mod db;

use serde::{Serialize, Deserialize};

#[allow(unused_imports)]
use rand::{thread_rng, Rng};
// use simple_logger::SimpleLogger;
#[allow(dead_code)]
#[allow(unused_imports)]
use std::thread;
use std::time::Duration;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use std::collections::hash_map::Entry;

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

#[cfg(target_os = "windows")]
type PeripheralImp = btleplug::winrtble::peripheral::Peripheral;


use btleplug::api::{CentralEvent,BDAddr,PeripheralProperties};


pub const DBADDR_ZERO :BDAddr =  BDAddr {
    address : [0,0,0,0,0,0]
};
pub const DBADDR_MAX :BDAddr =  BDAddr {
    address : [0xFF,0xFF,0xFF,0xFF,0xFF,0xFF]
};    

#[cfg(target_os = "linux")]
fn print_adapter_info(adapter: &Adapter) {
    log::trace!(
        "connected adapter {:?} is powered: {:?}",
        adapter.name(),
        adapter.is_powered()
    );
}


fn match_filter(filter : BDAddr, address_to_log : &BDAddr ) -> bool {
    return filter == DBADDR_MAX || filter == *address_to_log;
}


#[cfg(any(target_os = "windows", target_os = "macos"))]
fn print_adapter_info(_adapter: &Adapter) {
    log::trace!("adapter info can't be printed on Windows 10 or mac");
}

pub use super::core::*;

use async_trait::async_trait;

#[derive(Deserialize,Serialize)]
pub struct SourceBLEConfig {
    id : String,
}

pub struct SourceBLE {
    #[allow(dead_code)]
    config : Box<SourceBLEConfig>,
    name: String,
}

pub fn get_bytes_as_hex(bytes : &[u8]) -> String {
    let strings : Vec<String> = bytes.into_iter().map( |byte|format!("{:02X}", byte)).collect();
    return strings.join(":");
}

fn format_bytes(input_bytes : &[u8]) -> String {
    match std::str::from_utf8(input_bytes) {
        Ok(input_string) => {
            let chars_printable = input_string.chars().filter( |x| (*x as u32) > 15).count();
            if input_string.len() == 0 {
                return "(empty)".to_string();
            }
            let printable_percent = 100*chars_printable / input_string.len();
            if printable_percent > 90 {
                input_string.to_string()
            } else if printable_percent > 20 && input_string.len() < 20 {
                return format!("{} (\"{}\")",&get_bytes_as_hex(input_bytes),input_string.to_string());
                //"\"".to_string + input_string.to_string() + "\" (" + &get_bytes_as_hex(input_bytes) + ")"
            } else {
                get_bytes_as_hex(input_bytes)
            }
            //println!("    String : {}",input_string.escape_default());
        }
        Err(_) => get_bytes_as_hex(input_bytes)
    }
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

#[derive(Default)]
pub struct KnownDevice {
}

pub struct DeviceDB {
    pub devices :HashMap<BDAddr, KnownDevice>
}

impl DeviceDB {
    fn see_device(&mut self, addr : BDAddr ) -> &KnownDevice {
        let ent = match self.devices.entry(addr) {
            Entry::Occupied(o) => {o.into_mut()},
            Entry::Vacant(v) => { 
                log::info!("DeviceDiscovered: {:?}", addr);
                v.insert( KnownDevice::default())
            }
        };
        return ent;
    }
    

}


pub struct BleManager {
    #[allow(dead_code)]
    manager : Manager,
    #[allow(dead_code)]
    adapter : Adapter,
    #[allow(dead_code)]
    bluetooth_db : Arc<db::BluetoothDB>,
    #[allow(dead_code)]
    devices : Arc<Mutex<DeviceDB>>,
    //#[allow(dead_code)]
    poller : Option<std::thread::JoinHandle<()>>,
    receiver : crossbeam_channel::Receiver<CentralEvent>
}

impl BleManager {

    pub fn create() -> BleManager {
        log::info!("Initialising bluetooth");

        let bluetooth_db = Arc::new(db::BluetoothDB::create());

        let devices = Arc::new(Mutex::new(DeviceDB { devices: HashMap::new()}));

        let manager = Manager::new().unwrap();
        let adapter_list : Vec<Adapter> = manager.adapters().unwrap();
        
        log::trace!("Adapters : {}", adapter_list.len() );

        let adapter = adapter_list.into_iter().nth(0).unwrap();

        print_adapter_info(&adapter);

        let event_receiver = adapter.event_receiver().unwrap();

        //let db_ref = Arc::clone(&bluetooth_db);
        //let devices_ref = Arc::clone(&devices);
        
        let (ble_sender, ble_receiver) = crossbeam_channel::bounded(100);

        let poller = thread::spawn( move || {

            log::info!("Bluetooth Poller started");
            while let Ok(event) = event_receiver.recv() {
                ble_sender.send(event).unwrap();
            }            
            log::info!("Bluetooth Poller finished");
        });



        //let adapter = adapter_list.remove(0);
        return BleManager {
            manager,
            devices,
            adapter,
            receiver : ble_receiver,
            bluetooth_db : bluetooth_db,
            poller: Some(poller)

        };
    }

    pub fn shutdown(&mut self) -> Option<std::thread::JoinHandle<()>> {
        log::trace!("Terminating bluetooth (only we don't know how)");
        //What do we do here??!?
        return self.poller.take();
    }


    pub fn handle_event(&mut self, event : &CentralEvent, filter : BDAddr) -> () {

        match event {
            CentralEvent::DeviceDiscovered(address) => {
                //log::trace!("DeviceDiscovered: {:?}", address);
                self.devices.lock().unwrap().see_device(*address);
                    
            }
            CentralEvent::DeviceConnected(address) => {
                if match_filter(filter, address ) {
                    log::info!("DeviceConnected: {:?}", address);
                }
            }
            CentralEvent::DeviceDisconnected(address) => {
                if match_filter(filter, address ) {
                    log::info!("DeviceDisconnected: {:?}", address);
                }
            }
            CentralEvent::DeviceUpdated(address) => {
                if match_filter(filter, address ) {
                    log::info!("DeviceUpdated: {:?}", address);
                }
            }            
            CentralEvent::ManufacturerDataAdvertisement {
                address,
                manufacturer_id,
                data,
            } => {
                if match_filter(filter, address ) {
                    log::info!(
                        "ManufacturerDataAdvertisement: {:?}, {} {}, {}",
                        address, manufacturer_id, self.bluetooth_db.get_company( *manufacturer_id ), get_bytes_as_hex(data)
                    );
                }
                self.devices.lock().unwrap().see_device(*address);

            }
            CentralEvent::ServiceDataAdvertisement {
                address,
                service,
                data,
            } => {
                if match_filter(filter, address ) {
                    log::info!(
                        "ServiceDataAdvertisement: {:?}, {}, {:x?}",
                        address,
                        service.to_string(),
                        data
                    );
                }
            }
            CentralEvent::ServicesAdvertisement { address, services } => {
                if match_filter(filter, address ) {
                    let services: Vec<String> =
                        services.into_iter().map(|s| s.to_string()).collect();
                    log::info!("ServicesAdvertisement: {:?}, {:?}", address, services);
                }
            }
            e => {
                log::trace!("Event recevied {:?}",e);
            }
        }

    }



    pub fn scan(&mut self, ctrl_channel : crossbeam_channel::Receiver<()>, duration : Duration) {
        log::trace!("Doing scan for {:?} ...", duration);
        self.adapter.start_scan().unwrap();

        let ticks = crossbeam_channel::tick(duration);

        loop {
            crossbeam_channel::select! {
                recv(ticks) -> _ => {
                    log::trace!("Finished waiting");
                    break;
                }
                recv(ctrl_channel) -> _ => {
                    log::trace!("Aborting due to Ctrl-C!");
                    break;
                }
                recv(self.receiver) -> event => {
                    self.handle_event( &event.unwrap() , DBADDR_ZERO  );
                }

            }
        }
        log::trace!("Done");
    }

    pub async fn connect_and_print_characteristics(&mut self, ctrl_channel : crossbeam_channel::Receiver<()>, address_to_find: BDAddr) {

        let peripheral = match self.connect( ctrl_channel, address_to_find ).await {
            None => {
                println!("Unable to find device");
                return;
            }
            Some(p) => p
        };
        
        println!("Connected to:");
        self.print_peripheral(&peripheral);      

        println!("Characteristics:");
        let characteristics = peripheral.discover_characteristics().unwrap();
        for characteristic in characteristics.iter() {
            //let ch : &Characteristic = characteristic;
            self.print_characteristic(characteristic, Some(&peripheral) );
        }

    }


    pub async fn connect(&mut self, ctrl_channel : crossbeam_channel::Receiver<()>, address_to_find: BDAddr) -> Option<PeripheralImp> {

        log::trace!("Looking for device {} ...", address_to_find.to_string());

        //let address_to_find : BDAddr = address_to_find.parse().unwrap();

        self.adapter.start_scan().unwrap();

        loop {
            crossbeam_channel::select! {
                recv(ctrl_channel) -> _ => {
                    log::trace!("Aborting due to Ctrl-C!");
                    break;
                }
                recv(self.receiver) -> event => {
                    let event = event.unwrap();
                    self.handle_event( &event, address_to_find );
                    match event {
                        CentralEvent::DeviceDiscovered(address) => {
                            if match_filter(address_to_find, &address ) {
                                println!("******* Found {:0}, waiting before connecting!", address);
                                async_std::task::sleep(Duration::from_secs(2)).await;
                                let peripheral = self.adapter.peripheral(address).unwrap();

                                println!("******* Connecting to {} !", address);

                                match peripheral.connect() {
                                    Ok(result) => {println!("Connecting : {:?}", result);}
                                    Err(e) => {println!("Unable to start connection {:?}",e);}
                                };
                            }
                        },
                        CentralEvent::DeviceConnected(address) => {
                            if match_filter(address_to_find, &address ) {
                                println!("******* Connected to {:0}, fetching characteristics", address);
                                let peripheral : PeripheralImp = self.adapter.peripheral(address).unwrap();
                                return Some(peripheral);
                            }
                        },
                        CentralEvent::DeviceUpdated(address) => {
                            if match_filter(address_to_find, &address ) {
                                println!("******* Updated {:0}", address);
                            }
                        },
                        _ => ()
                    }
                }
            }
        }

        self.list( address_to_find );
        return None;

    }



    pub fn print_characteristic(&self, characteristic : &Characteristic, optional_peripheral : Option<&PeripheralImp> ) {
        let name = self.bluetooth_db.get_characteristic_name(characteristic.uuid);

        match optional_peripheral {
            Some(peripheral) => {
                match peripheral.read( characteristic ) {
                    Ok(bytes) => {
                        let value_formatted = format_bytes(&bytes);
                        println!("    {} = {}", name, value_formatted);
                    }
                    Err(e) => {
                        println!("    {} : Error:{:?}", name,e);
                    }
                }
            }
            None => {
            }
        }
    }

    pub fn print_peripheral(&self, peripheral : &PeripheralImp ) {
        let p : &PeripheralProperties = &peripheral.properties();//.local_name,
        //let x : dyn Peripheral = peripheral;
        println!(
            "{}  ({:?}, tx_power_level:{})", 
            peripheral.address(),
            p.address_type,
            match p.tx_power_level { Some(x) => x.to_string(), None => "?".to_string()}
        );

        if let Some(n) = &p.local_name {
            println!( "  Name : \"{}\"",n);
        }

        match &p.local_name { Some(x) => x, None => "?"};


        for (id, data) in &p.manufacturer_data {
            println!( "  Manufacturer Data {} ({})  {:?}",id,self.bluetooth_db.get_company(*id), get_bytes_as_hex(data));
        }

        for (uuid, data) in &p.service_data {
            let name = self.bluetooth_db.get_service_name(*uuid);
            println!( "  Service Data {} ({})  {:?}",uuid,name, get_bytes_as_hex(data));
        }
        for uuid in &p.services {
            println!( "  Service {} ({})",uuid,self.bluetooth_db.get_service_name(*uuid));
        }
        let characteristics : std::collections::BTreeSet<btleplug::api::Characteristic> = peripheral.characteristics();
        println!("  Char length : {:?}",characteristics.len());
        for characteristic in characteristics.iter() {
            self.print_characteristic(characteristic, Option::None);
            //println!("  Characteristics : {:?}",q);
        }
    }
    
    pub fn list(&mut self, address_to_find: BDAddr) {
        let peripherals = self.adapter.peripherals();
        for peripheral in peripherals.iter() {
            if match_filter(address_to_find, &peripheral.address()) {
                self.print_peripheral( peripheral );
            }

        }

        println!("Done");
    }

}

