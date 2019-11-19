mod cwasm;

// use wabt::{wat2wasm, wasm2wat};
use cwasm::CWasm;
use std::fs::File;
use std::io::{prelude::*};

fn main() {
    let fname = "toys/math.wasm";
    let mut f = File::open(fname).expect("Could not open wasm file");
    let mut buff = Vec::new();
    f.read_to_end(&mut buff).expect("Could not read file");
    println!("Read wasm file with size {}", buff.len());
    let c = CWasm::parse_wasm(&buff);
    println!("{:?}", c);
}
