use byteorder::{LittleEndian, ReadBytesExt};
use hex::encode;
use leb128::*;
use std::io::{prelude::*, Cursor};

#[derive(Debug)]
enum SectionID {
    CustomSection = 0,
    TypeSection = 1,
    ImportSection = 2,
    FunctionSection = 3,
    TableSection = 4,
    MemorySection = 5,
    GlobalSection = 6,
    ExportSection = 7,
    StartSection = 8,
    ElementSection = 9,
    CodeSection = 10,
    DataSection = 11,
    InvalidSection,
}

impl SectionID {
    fn from_u8(value: u8) -> SectionID {
        match value {
            0 => SectionID::CustomSection,
            1 => SectionID::TypeSection,
            2 => SectionID::ImportSection,
            3 => SectionID::FunctionSection,
            4 => SectionID::TableSection,
            5 => SectionID::MemorySection,
            6 => SectionID::GlobalSection,
            7 => SectionID::ExportSection,
            8 => SectionID::StartSection,
            9 => SectionID::ElementSection,
            10 => SectionID::CodeSection,
            11 => SectionID::DataSection,
            _ => SectionID::InvalidSection,
        }
    }
}

#[derive(Debug)]
enum ValueType {
    F64 = 0x7C,
    F32 = 0x7D,
    I64 = 0x7E,
    I32 = 0x7F,
    Invalid,
}

impl ValueType {
    fn from_u8(value: u8) -> ValueType {
        match value {
            0x7C => ValueType::F64,
            0x7D => ValueType::F32,
            0x7E => ValueType::I64,
            0x7F => ValueType::I32,
            _ => ValueType::Invalid,
        }
    }
}

struct Function {
    params: Vec<ValueType>,
    returns: Vec<ValueType>,
}

pub struct CWasm {
    functions: Vec<Function>,
}

impl CWasm {
    pub fn parse_wasm(binfile: &[u8]) -> CWasm {
        let mut cur = Cursor::new(binfile);
        // First read the magic bytes
        let mut magic = [0u8; 4];
        cur.read_exact(&mut magic).expect("Could not read magic");
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
                SectionID::TypeSection => CWasm::parse_section_type(&mut cur),
                SectionID::FunctionSection => CWasm::parse_section_function(&mut cur),
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
        CWasm {
            functions: Vec::new(),
        }
    }

    // The type section has the id 1. It decodes into a vector of function types
    // that represent the types component of a module.
    fn parse_section_type(cur: &mut Cursor<&[u8]>) {
        let section_size = leb128::read::unsigned(cur).expect("could not get size in Type section");
        println!("\tsection Type is {} bytes", section_size);
        let num_functions =
            leb128::read::unsigned(cur).expect("could not get numfuncs in Type section");
        println!("\tsection Type contains {} functions", num_functions);
        let mut functions = Vec::<Function>::with_capacity(num_functions as usize);
        for _ in 0..num_functions {
            let func_byte = cur.read_u8().unwrap();
            if func_byte == 0x60 {
                let num_params = leb128::read::unsigned(cur).expect("could not get param num");
                let mut params = Vec::<ValueType>::with_capacity(num_params as usize);
                for _ in 0..num_params {
                    let param_type = cur.read_u8().unwrap();
                    params.push(ValueType::from_u8(param_type));
                    println!("\t\tparam of type {:?} ({:X})", &params.last().unwrap(), param_type);
                }
                let num_results = leb128::read::unsigned(cur).expect("could not get result num");
                let mut returns = Vec::<ValueType>::with_capacity(num_results as usize);
                for _ in 0..num_results {
                    let resul_type = cur.read_u8().unwrap();
                    returns.push(ValueType::from_u8(resul_type));
                    println!("\t\treturn of type {:?} ({:X})", &params.last().unwrap(), resul_type);
                }
                let func = Function {
                    params,
                    returns
                };
                functions.push(func);
            } else {
                println!("Encountered invalid func byte {}", func_byte);
                break;
            }
        }
    }

    fn parse_section_function(cur: &mut Cursor<&[u8]>) {}
}
