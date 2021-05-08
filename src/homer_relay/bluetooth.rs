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
use btleplug::api::{CentralEvent,BDAddr};


#[cfg(target_os = "linux")]
fn print_adapter_info(adapter: &Adapter) {
    log::trace!(
        "connected adapter {:?} is powered: {:?}",
        adapter.name(),
        adapter.is_powered()
    );
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
    #[allow(dead_code)]
    poller : Option<std::thread::JoinHandle<()>>
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

        let db_ref = Arc::clone(&bluetooth_db);
        let devices_ref = Arc::clone(&devices);
        let poller = thread::spawn( move || {
        log::info!("Bluetooth Poller started");
            while let Ok(event) = event_receiver.recv() {
                match event {
                    CentralEvent::DeviceDiscovered(address) => {
                        //log::trace!("DeviceDiscovered: {:?}", address);
                        devices_ref.lock().unwrap().see_device(address);
                            
                    }
                    CentralEvent::DeviceConnected(address) => {
                        log::trace!("DeviceConnected: {:?}", address);
                    }
                    CentralEvent::DeviceDisconnected(address) => {
                        log::trace!("DeviceDisconnected: {:?}", address);
                    }
                    CentralEvent::ManufacturerDataAdvertisement {
                        address,
                        manufacturer_id,
                        data,
                    } => {
                        log::trace!(
                            "ManufacturerDataAdvertisement: {:?}, {} {}, {:x?}",
                            address, manufacturer_id, db_ref.get_company( manufacturer_id ), data
                        );
                        devices_ref.lock().unwrap().see_device(address);

                    }
                    CentralEvent::ServiceDataAdvertisement {
                        address,
                        service,
                        data,
                    } => {
                        log::trace!(
                            "ServiceDataAdvertisement: {:?}, {}, {:x?}",
                            address,
                            service.to_string(),
                            //service.to_short_string(),
                            data
                        );
                    }
                    CentralEvent::ServicesAdvertisement { address, services } => {
                        let services: Vec<String> =
                            services.into_iter().map(|s| s.to_string()).collect();
                        log::trace!("ServicesAdvertisement: {:?}, {:?}", address, services);
                    }                    
                    e => {
                        log::trace!("Event recevied {:?}",e);
                    }
                }
            }            
            log::info!("Bluetooth Poller finished");
        });

        //let adapter = adapter_list.remove(0);
        return BleManager {
            manager,
            devices,
            adapter,
            bluetooth_db : bluetooth_db,
            poller: Some(poller)

        };
    }

    pub fn shutdown(&mut self) -> Option<std::thread::JoinHandle<()>> {
        log::trace!("Terminating bluetooth (only we don't know how)");
        //What do we do here??!?
        return self.poller.take();
    }



    pub fn scan(&mut self, duration : Duration , ctrl_channel : crossbeam_channel::Receiver<()>) {
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
            }
        }
        log::trace!("Done");
    }

    pub fn list(&mut self) {
        for peripheral in self.adapter.peripherals().iter() {
            let p = peripheral.properties();//.local_name,
            println!(
                "{} Power:{}", 
                peripheral.address(),
                match p.tx_power_level { Some(x) => x.to_string(), None => "?".to_string()}
            );

            if let Some(n) = &p.local_name {
                println!( "  Name : \"{}\"",n);
            }

            match &p.local_name { Some(x) => x, None => "?"};


            for (id, data) in p.manufacturer_data {
                println!( "  Manufacturer Data {} ({})  {:?}",id,self.bluetooth_db.get_company(id), hex::encode(data));
            }

            for (uuid, data) in p.service_data {
                let name = self.bluetooth_db.get_service_name(uuid);
                println!( "  Service Data {} ({})  {:?}",uuid,name, hex::encode(data));
            }
            for uuid in p.services {
                println!( "  Service {} ({})",uuid,self.bluetooth_db.get_service_name(uuid));
            }
            
            let properties : btleplug::api::PeripheralProperties = peripheral.properties();
            println!("Properties : {:?}",properties);

            let characteristics : std::collections::BTreeSet<btleplug::api::Characteristic> = peripheral.characteristics();
            println!("  Char length : {:?}",characteristics.len());
            for q in characteristics.iter() {
                println!("  Characteristics : {:?}",q);
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

