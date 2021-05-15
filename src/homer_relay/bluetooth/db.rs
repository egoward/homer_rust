use serde::Deserialize;
use uuid::Uuid;

#[derive(Deserialize)]
struct CompanyJSON {
    code: u16,
    name: String,
}   

#[derive(Deserialize)]
struct BluetoothMetadata {
    #[allow(dead_code)]
    name: String,
    #[allow(dead_code)]
    identifier: String,
    uuid: String,
    #[allow(dead_code)]
    source: String
}

pub struct BluetoothDB {
    map_company : std::collections::HashMap<u16, String>,
    #[allow(dead_code)]
    map_characteristic : std::collections::HashMap<Uuid, BluetoothMetadata>,
    #[allow(dead_code)]
    map_service : std::collections::HashMap<Uuid, BluetoothMetadata>,
    #[allow(dead_code)]
    map_descriptor : std::collections::HashMap<Uuid, BluetoothMetadata>,

}

const BTLT_UUID_D2 : u16 = 0;
const BTLT_UUID_D3 : u16 = 0x1000;
const BTLT_UUID_D4 : [u8;8]= [0x80,0x00,0x00,0x80,0x5F,0x9B,0x34,0xFB];

impl BluetoothDB {

    fn read_name_code_file(filename : &str)-> std::collections::HashMap<u16, String> {
        log::trace!("Parsing {}", filename);
        let file = std::fs::File::open(filename).unwrap();
        let json : Vec<CompanyJSON> = serde_json::from_reader(file).unwrap();
        return json.into_iter().map( |x| (x.code, x.name)).collect();
    }


    fn parse_uuid( string : &str) -> Uuid {
        if string.len() == 4 {
            let d1 = u32::from_str_radix(string, 16).unwrap();
            let ret = Uuid::from_fields( d1,BTLT_UUID_D2,BTLT_UUID_D3,&BTLT_UUID_D4).unwrap();
            return ret;
        } else if string.len() == 36  {
            return Uuid::parse_str( string ).unwrap();
        } else {
            panic!("Unexpected length of uuid {} ({})" , string, string.len())
        }
    }

    fn read_descriptor_file(filename : &str) -> std::collections::HashMap<Uuid, BluetoothMetadata> {
        log::trace!("Parsing {}", filename);
        let file = std::fs::File::open(filename).unwrap();
        let json : Vec<BluetoothMetadata> = serde_json::from_reader(file).unwrap();
        let ret : std::collections::HashMap<Uuid, BluetoothMetadata> = json.into_iter().map( |x| 
            (BluetoothDB::parse_uuid(&x.uuid), x ) 
        ).collect();
        return ret;
    }

    pub fn create() -> BluetoothDB {
        return BluetoothDB {
            map_company : BluetoothDB::read_name_code_file("data/bluetooth-numbers-database/v1/company_ids.json"),
            map_characteristic : BluetoothDB::read_descriptor_file("data/bluetooth-numbers-database/v1/characteristic_uuids.json"),
            map_service : BluetoothDB::read_descriptor_file("data/bluetooth-numbers-database/v1/service_uuids.json"),
            map_descriptor : BluetoothDB::read_descriptor_file("data/bluetooth-numbers-database/v1/descriptor_uuids.json"),
        }
    }

    pub fn get_company(&self, id : u16) -> String {
        match self.map_company.get(&id) {
            Some(v) => {return v.clone();}
            None => {return format!("Unknown({})",id)}
        }
    }
    pub fn get_service_name(&self, uuid : Uuid) -> &str {
        match self.map_service.get(&uuid) {
            Some(v) => {return &v.name;}
            None => {"Unknown"}
        }
    }
    pub fn get_characteristic_name(&self, uuid : Uuid) -> String {
        match self.map_characteristic.get(&uuid) {
            Some(v) => {return v.name.to_string();}
            None => {
                match uuid.as_fields() {
                    (a,BTLT_UUID_D2,BTLT_UUID_D3,&BTLT_UUID_D4) => {
                        format!("BTLE UUID {:#04x}",a)
                    }
                    _ => {
                        format!("UUID {}", uuid)
                    }
                }
                //(a,b,c,d) = uuid.as_Fields();

            }
        }
    }        

}
