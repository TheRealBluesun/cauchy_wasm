use byteorder::{LittleEndian, ReadBytesExt};
use hex::encode;
use leb128::*;
use std::io::{prelude::*, Cursor};

#[derive(Debug)]
enum SectionID {
    Custom_Section = 0,
    Type_Section = 1,
    Import_Section = 2,
    Function_Section = 3,
    Table_Section = 4,
    Memory_Section = 5,
    Global_Section = 6,
    Export_Section = 7,
    Start_Section = 8,
    Element_Section = 9,
    Code_Section = 10,
    Data_Section = 11,
    Invalid_Section,
}

impl SectionID {
    fn from_u8(value: u8) -> SectionID {
        match value {
            0 => SectionID::Custom_Section,
            1 => SectionID::Type_Section,
            2 => SectionID::Import_Section,
            3 => SectionID::Function_Section,
            4 => SectionID::Table_Section,
            5 => SectionID::Memory_Section,
            6 => SectionID::Global_Section,
            7 => SectionID::Export_Section,
            8 => SectionID::Start_Section,
            9 => SectionID::Element_Section,
            10 => SectionID::Code_Section,
            11 => SectionID::Data_Section,
            _ => SectionID::Invalid_Section,
        }
    }
}

enum ValueType {}

pub struct CWasm {}

impl CWasm {
    pub fn parse_wasm(binfile: &[u8]) {
        let mut cur = Cursor::new(binfile);
        // First read the magic bytes
        let mut magic = [0u8; 4];
        cur.read(&mut magic).expect("Could not read magic");
        assert_eq!(&magic, b"\0asm");
        // // Then read the version
        let version = cur
            .read_u32::<LittleEndian>()
            .expect("Failed to parse version");
        // Then read the first section ID
        while let Ok(id_byte) = cur.read_u8() {
            let id = SectionID::from_u8(id_byte);
            println!("found section {:?} ({:X})", id, id_byte);
            match id {
                SectionID::Type_Section => CWasm::parse_section_type(&mut cur),
                _ => {
                    println!("No method to parse section {:?}", id);
                    break;
                }
            }
        }

        // Print out the unparsed data
        let mut remaining = Vec::new();
        cur.read_to_end(&mut remaining)
            .expect("Could not read unparsed data");
        println!("Buff left: {}", hex::encode(remaining));
    }

    // The type section has the id 1. It decodes into a vector of function types
    // that represent the types component of a module.
    fn parse_section_type(cur: &mut Cursor<&[u8]>) {
        let section_size = leb128::read::unsigned(cur).expect("could not get size in Type section");
        println!("\tsection Type is {} bytes", section_size);
        let num_functions =
            leb128::read::unsigned(cur).expect("could not get numfuncs in Type section");
        println!("\tsection Type contains {} functions", num_functions);
        for _ in 0..num_functions {
            let func_byte = cur.read_u8().unwrap();
            if func_byte == 0x60 {
                let num_params = leb128::read::unsigned(cur).expect("could not get param num");
                for _ in 0..num_params {
                    let param_type = cur.read_u8().unwrap();
                    println!("\t\tparam of type {:X}", param_type);
                }
                let num_results = leb128::read::unsigned(cur).expect("could not get result num");
                for _ in 0..num_results {
                    let resul_type = cur.read_u8().unwrap();
                    println!("\t\treturn of type {:X}", resul_type);
                }
            } else {
                println!("Encountered invalid func byte {}", func_byte);
                break;
            }
        }
        // Function types are encoded by the byte 0x60 followed by the
        // respective vectors of parameter and result types

        // if let Some(mut buf) = CWasm::parse_vector(cur) {
        //     println!("contents of section Type: {}", hex::encode(&buf));
        //     // buf now contains the vector of function types
        //     let num_functions = leb128::read::unsigned(&buf);
        // } else {
        //     println!("Unable to parse section Type");
        // }
    }

    fn parse_vector(cur: &mut Cursor<&[u8]>) -> Option<Vec<u8>> {
        let size = leb128::read::unsigned(cur).expect("Could not decode vector size") as usize;
        let mut buf = Vec::with_capacity(size);
        let bytes_read = cur
            .take(size as u64)
            .read_to_end(&mut buf)
            .expect("unable to read vector data");
        if bytes_read == size {
            Some(buf)
        } else {
            None
        }
    }
}
