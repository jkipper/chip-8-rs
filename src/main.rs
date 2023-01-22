use std::{fs, io::Read, path::Path};

#[derive(Debug)]
struct Address(u16);

impl From<&[u8; 3]> for Address {
    fn from(addr: &[u8; 3]) -> Self {
        Address((addr[0] as u16) << 8 | (addr[1] as u16) << 4 | addr[2] as u16)
    }
}

impl From<usize> for Address {
    fn from(value: usize) -> Self {
        if value > 0xFFF {
            panic!("Address {:?} is too large", value);
        }
        Address(value as u16)
    }
}

impl From<Address> for usize {
    fn from(value: Address) -> Self {
        value.0 as Self
    }
}

#[derive(Debug)]
struct Const8(u8);

impl From<&[u8; 2]> for Const8 {
    fn from(data: &[u8; 2]) -> Self {
        Const8(data[0 << 4] | data[1])
    }
}

#[derive(Debug)]
struct Const4(u8);
#[derive(Debug)]
struct Var(u8);

#[derive(Debug)]
enum OpCode {
    CallRoutine(Address),     // 0
    Clear(),                  // 0
    Ret(),                    // 0
    Jmp(Address),             // 1
    CallSubroutine(Address),  // 2
    SkipConstEq(Var, Const8), // 3
    SkipConstNe(Var, Const8), // 4
    SkipEq(Var, Var),         // 5
    SetConst(Var, Const8),    // 6
    AddConst(Var, Const8),    // 7
    Set(Var, Var),            // 8
    BitOR(Var, Var),          // 8
    BitAND(Var, Var),         // 8
    BitXOR(Var, Var),         // 8
    AddEqReg(Var, Var),       // 8
    SubEqReg(Var, Var),       // 8
    BitRShift(Var, Var),      // 8
    SubReg(Var, Var),         // 8
    BitLShift(Var, Var),      // 8
    SkipNe(Var, Var),         // 9
    IToAddr(Address),         // A
    SetPc(Address),           // B
    Rand(Var, Const8),        // C
    Draw(Var, Var, Const4),   // D
    KeyEq(Var),               // E
    KeyNe(Var),               // E
    GetTimer(Var),            // F
    AwaitKey(Var),            // F
    SetDelayTimer(Var),       // F
    SetSoundTimer(Var),       // F
    IAdd(Var),                // F
    ISetSprite(Var),          // F
    StoreBCD(Var),            // F
    RegDump(Var),             // F
    RegLoad(Var),             // F
}

struct SystemState {
    memory: Memory,
    registers: [u8; 16],
    pc: usize,
    sp: usize,
}

struct Memory {
    data: [u8; 4096],
}
impl Memory {
    fn new() -> Memory {
        Memory { data: [0; 4096] }
    }

    fn load_program(&mut self, program: &Path) {
        let _read_bytes = fs::File::open(program)
            .expect("File should exist")
            .read(&mut self.data[0x200..])
            .expect("Failed to load into memory");
    }

    fn fetch_opcode(&self, address: Address) -> Option<OpCode> {
        let idx: usize = address.into();
        let opc = &self.data[idx..idx + 2];
        let parts = [
            (opc[0] & 0xF0) >> 4,
            opc[0] & 0x0F,
            (opc[1] & 0xF0) >> 4,
            opc[1] & 0x0F,
        ];
        return match parts {
            [0x0, 0x0, 0xE, 0xE] => Some(OpCode::Ret()),
            [0x0, 0x0, 0xE, 0x0] => Some(OpCode::Clear()),
            [0x0, ref addr @ ..] => Some(OpCode::CallRoutine(addr.into())),
            [0x1, ref addr @ ..] => Some(OpCode::Jmp(addr.into())),
            [0x2, ref addr @ ..] => Some(OpCode::CallSubroutine(addr.into())),
            [0x3, x, ref rest @ ..] => Some(OpCode::SkipConstEq(Var(x), rest.into())),
            [0x4, x, ref rest @ ..] => Some(OpCode::SkipConstNe(Var(x), rest.into())),
            [0x5, x, y, 0x0] => Some(OpCode::SkipEq(Var(x), Var(y))),
            [0x6, x, ref rest @ ..] => Some(OpCode::SetConst(Var(x), rest.into())),
            [0x7, x, ref rest @ ..] => Some(OpCode::AddConst(Var(x), rest.into())),
            [0x8, x, y, 0x0] => Some(OpCode::Set(Var(x), Var(y))),
            [0x8, x, y, 0x1] => Some(OpCode::BitOR(Var(x), Var(y))),
            [0x8, x, y, 0x2] => Some(OpCode::BitAND(Var(x), Var(y))),
            [0x8, x, y, 0x3] => Some(OpCode::BitXOR(Var(x), Var(y))),
            [0x8, x, y, 0x4] => Some(OpCode::AddEqReg(Var(x), Var(y))),
            [0x8, x, y, 0x5] => Some(OpCode::SubEqReg(Var(x), Var(y))),
            [0x8, x, y, 0x6] => Some(OpCode::BitRShift(Var(x), Var(y))),
            [0x8, x, y, 0x7] => Some(OpCode::SubReg(Var(x), Var(y))),
            [0x8, x, y, 0xE] => Some(OpCode::BitLShift(Var(x), Var(y))),
            [0x9, x, y, 0x0] => Some(OpCode::SkipNe(Var(x), Var(y))),
            [0xA, ref addr @ ..] => Some(OpCode::IToAddr(addr.into())),
            [0xB, ref addr @ ..] => Some(OpCode::SetPc(addr.into())),
            [0xC, x, ref c8 @ ..] => Some(OpCode::Rand(Var(x), c8.into())),
            [0xD, x, y, c] => Some(OpCode::Draw(Var(x), Var(y), Const4(c))),
            [0xE, x, 0x9, 0xE] => Some(OpCode::KeyEq(Var(x))),
            [0xE, x, 0xA, 0x1] => Some(OpCode::KeyNe(Var(x))),
            [0xF, x, 0x0, 0x7] => Some(OpCode::GetTimer(Var(x))),
            [0xF, x, 0x1, 0xA] => Some(OpCode::AwaitKey(Var(x))),
            [0xF, x, 0x1, 0x5] => Some(OpCode::SetDelayTimer(Var(x))),
            [0xF, x, 0x1, 0x8] => Some(OpCode::SetSoundTimer(Var(x))),
            [0xF, x, 0x1, 0xE] => Some(OpCode::IAdd(Var(x))),
            [0xF, x, 0x2, 0x9] => Some(OpCode::ISetSprite(Var(x))),
            [0xF, x, 0x3, 0x3] => Some(OpCode::StoreBCD(Var(x))),
            [0xF, x, 0x5, 0x5] => Some(OpCode::RegDump(Var(x))),
            [0xF, x, 0x6, 0x5] => Some(OpCode::RegLoad(Var(x))),
            _ => {
                println!("Unkown opcode for {:x?}", parts);
                None
            }
        };
    }
}

impl SystemState {
    fn new() -> SystemState {
        SystemState {
            memory: Memory::new(),
            registers: [0; 16],
            pc: 0x200,
            sp: 0,
        }
    }
}

fn main() {
    let mut state = SystemState::new();
    let test_path = Path::new("../chip8-test-rom/chip8-test-rom.ch8");
    state.memory.load_program(test_path);
    println!(
        "first opcode = {:x?}",
        state.memory.fetch_opcode(state.pc.into())
    );
}
