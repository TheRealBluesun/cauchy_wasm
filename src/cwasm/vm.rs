use super::*;
use std::io::{BufReader, Read};

enum StackItem {
    ValueType,
}

pub struct VM {
    stack: Vec<StackItem>,
}

impl VM {
    pub fn new() -> VM {
        VM {
            stack: Vec::<StackItem>::new(),
        }
    }
    pub fn run(&mut self, cwasm: &CWasm) {
        // For now, just execute the first function
        let mut code = cwasm.codes.first().unwrap();
        let mut rdr = BufReader::new(&code[..]);
        // First we get the number of locals in this func
        let num_locals = leb128::read::unsigned(&mut rdr).unwrap();
        let mut is_done = false;

        while !is_done {
            let mut instr = [0u8; 1];
            rdr.read_exact(&mut instr);
            println!("{:X?}", instr);
            self.handle_istr(instr[0], &mut rdr);
        }
    }

    fn handle_istr(&mut self, instr: u8, rdr: &mut BufReader<&[u8]>) {
        match instr {
            //local.get x
            0x20 => {
                // First we grab out x
                let mut x = [0u8; 1];
                let x = rdr.read_exact(&mut x);
            }
            //local.set x
            0x21 => {}
            //local.tee x
            0x22 => {}
            //global.get x
            0x23 => {}
            //global.set x
            0x24 => {}
            _ => {
                panic! {"Unhandled instruction {:X}", instr}
            }
        }
    }
}
