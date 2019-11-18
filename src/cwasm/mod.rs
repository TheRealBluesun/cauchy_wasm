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
            0xA => SectionID::CodeSection,
            0xB => SectionID::DataSection,
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

#[derive(Debug)]
enum ExportDesc {
    FuncIdx = 0,
    TableIdx = 1,
    MemIdx = 2,
    GlobalIdx = 3,
    InvalidIdx,
}

impl ExportDesc {
    fn from_u8(value: u8) -> ExportDesc {
        match value {
            0 => ExportDesc::FuncIdx,
            1 => ExportDesc::TableIdx,
            2 => ExportDesc::MemIdx,
            3 => ExportDesc::GlobalIdx,
            _ => ExportDesc::InvalidIdx,
        }
    }
}

struct Export {
    name: String,
    desc: ExportDesc,
    idx: u32,
}

struct Function {
    params: Vec<ValueType>,
    returns: Vec<ValueType>,
}

pub struct CWasm {
    functions: Vec<Function>,
    exports: Vec<Export>,
    codes: Vec<Vec<u8>>,
    sections: Vec<SectionID>,
}

impl CWasm {
    pub fn parse_wasm(binfile: &[u8]) -> CWasm {
        let mut cur = Cursor::new(binfile);
        // First read the magic bytes and assert
        let mut magic = [0u8; 4];
        cur.read_exact(&mut magic).expect("Could not read magic");
        assert_eq!(&magic, b"\0asm");
        let _version = cur
            .read_u32::<LittleEndian>()
            .expect("Failed to parse version");
        let mut functions = Vec::new();
        let mut exports = Vec::new();
        let mut codes = Vec::new();
        // Then read the sections
        // Iterate through sections
        let mut sections = Vec::<SectionID>::new();
        while let Ok(id_byte) = cur.read_u8() {
            let id = SectionID::from_u8(id_byte);
            sections.push(id);
            let section_size = leb128::read::unsigned(&mut cur)
                .expect("could not get section size in section")
                as usize;
            println!("found section {:?} ({:X})", sections.last().unwrap(), id_byte);
            match sections.last().unwrap() {
                SectionID::TypeSection => {
                    functions = CWasm::parse_section_type(&mut cur, section_size);
                }
                SectionID::FunctionSection => {
                    CWasm::parse_section_function(&mut cur, section_size);
                }
                SectionID::ExportSection => {
                    exports = CWasm::parse_section_export(&mut cur, section_size);
                }
                SectionID::CodeSection => {
                    codes = CWasm::parse_section_code(&mut cur, section_size);
                }
                _ => {
                    println!("No method to parse section {:?}", sections.last().unwrap());
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
            functions,
            exports,
            codes,
            sections
        }
    }

    // The type section has the id 1. It decodes into a vector of function types
    // that represent the types component of a module.
    fn parse_section_type(cur: &mut Cursor<&[u8]>, size: usize) -> Vec<Function> {
        println!("\tsection Type is {} bytes", size);
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
                    println!(
                        "\t\tparam of type {:?} ({:X})",
                        &params.last().unwrap(),
                        param_type
                    );
                }
                let num_returns = leb128::read::unsigned(cur).expect("could not get num returns");
                let mut returns = Vec::<ValueType>::with_capacity(num_returns as usize);
                for _ in 0..num_returns {
                    let ret_type = cur.read_u8().unwrap();
                    returns.push(ValueType::from_u8(ret_type));
                    println!(
                        "\t\treturn of type {:?} ({:X})",
                        &params.last().unwrap(),
                        ret_type
                    );
                }
                functions.push(Function { params, returns });
            } else {
                println!("Encountered invalid func byte {}", func_byte);
                break;
            }
        }
        functions
    }

    // The function section has the id 3. It decodes into a vector of type indices that represent
    // the type fields of the functions in the funcs component of a module. The locals and body fields
    // of the respective functions are encoded separately in the code section.
    fn parse_section_function(cur: &mut Cursor<&[u8]>, size: usize) -> Vec<u32> {
        println!("\tsection Function is {} bytes", size);
        let mut retvec = Vec::<u32>::with_capacity(size);
        for _ in 0..size {
            let idx =
                leb128::read::unsigned(cur).expect("could not get index in func section") as u32;
            retvec.push(idx);
        }
        println!("\t{:X?}", &retvec);
        retvec
    }

    // The export section has the id 7. It decodes into a vector of exports that represent the exports component of a module.
    fn parse_section_export(cur: &mut Cursor<&[u8]>, size: usize) -> Vec<Export> {
        println!("\tsection Export is {} bytes", size);
        let num_entries = leb128::read::unsigned(cur).expect("could not get number of exports");
        println!("\tsection Export contains {} entries", num_entries);
        let mut exports = Vec::<Export>::with_capacity(num_entries as usize);
        for _ in 0..num_entries {
            let name = CWasm::read_str(cur);
            let export_desc = cur.read_u8().unwrap();
            let desc = ExportDesc::from_u8(export_desc);
            let idx = leb128::read::unsigned(cur).expect("could not get export idx") as u32;
            exports.push(Export { name, desc, idx });
        }
        exports
    }

    // The code section has the id 10. It decodes into a vector of code entries that are pairs of value type vectors
    // and expressions. They represent the locals and body field of the functions in the funcs component of a module.
    // The type fields of the respective functions are encoded separately in the function section.
    fn parse_section_code(cur: &mut Cursor<&[u8]>, size: usize) -> Vec<Vec<u8>> {
        println!("\tsection Code is {} bytes", size);
        let num_codes = leb128::read::unsigned(cur).expect("could not get code size") as usize;
        let mut retvec = Vec::<Vec<u8>>::with_capacity(num_codes);
        for _ in 0..num_codes {
            let code_size = leb128::read::unsigned(cur).expect("could not get code size") as usize;
            // let num_locals = leb128::read::unsigned(cur).expect("could not get numlocals") as usize;
            // let mut locals = Vec::<ValueType>::with_capacity(num_locals);
            // for _ in 0..num_locals {
            //     let valtype = ValueType::from_u8(cur.read_u8().unwrap());
            //     locals.push(valtype);
            // }
            let mut code = vec![0u8; code_size];
            cur.read_exact(&mut code)
                .expect("unable to read code from code section");
            retvec.push(code);
        }
        retvec
    }

    fn read_str(cur: &mut Cursor<&[u8]>) -> String {
        let size = leb128::read::unsigned(cur).expect("could not get string size");
        let mut buf = vec![0u8; size as usize];
        cur.read_exact(&mut buf).expect("unable to read string");
        String::from_utf8_lossy(&buf).to_string()
    }
}
