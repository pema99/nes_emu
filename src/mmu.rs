use ppu::Ppu;
use apu::Apu;
use mapper::Mapper;
use std::cell::RefCell;
use std::rc::Rc;
use controller::Controller;

const WRAM_START: u16 = 0x0000;
const WRAM_END: u16 = 0x1FFF;
const PPU_START: u16 = 0x2000;
const PPU_END: u16 = 0x3FFF;
const ROM_START: u16 = 0x4020;
const ROM_END: u16 = 0xFFFF;

pub struct Mmu {
    pub ppu: Ppu,
    pub apu: Apu,
    pub ram: Ram,
    pub mapper: Rc<RefCell<Mapper>>,
    pub ctrl0: Controller,
    pub ctrl1: Controller,
}

//#[derive(Serialize, Deserialize)]
pub struct Ram([u8; 0xFFF]);

impl Ram {
    pub fn new() -> Ram {
        Ram { 0: [0; 0xFFF] }
    }

    fn load(&self, address: u16) -> u8 {
        self.0[address as usize]
    }

    fn store(&mut self, address: u16, val: u8) {
        self.0[address as usize] = val;
    }
}

impl Mmu {
    pub fn new(
        apu: Apu,
        ram: Ram,
        ppu: Ppu,
        mapper: Rc<RefCell<Mapper>>,
    ) -> Mmu {
        Mmu {
            ppu: ppu,
            apu: apu,
            ram: ram,
            mapper: mapper,
            ctrl0: Controller::new(),
            ctrl1: Controller::new(),
        }
    }

    pub fn store(&mut self, address: u16, val: u8) {
        match address {
            WRAM_START...WRAM_END => self.ram.store(address & 0x7FF, val),
            PPU_START...PPU_END => self.ppu.store((address - 0x2000) & 7, val),
            0x4016 => {
                self.ctrl0.store(val);
                self.ctrl1.store(val);
            }
            0x4000...0x4017 => self.apu.store(address - 0x4000, val),
            0x4018...0x401F => println!("disabled normally"),
            ROM_START...ROM_END => {
                self.mapper.borrow_mut().store_prg(address, val)
            }
        }
    }

    pub fn ld8(&mut self, address: u16) -> u8 {
        match address {
            WRAM_START...WRAM_END => self.ram.load(address & 0x7FF),
            PPU_START...PPU_END => self.ppu.ld((address - 0x2000) & 7),
            0x4015 => self.apu.load(address - 0x4000),
            0x4016 => self.ctrl0.ld8(),
            0x4017 => self.ctrl1.ld8(),
            0x4000...0x4014 | 0x4018...0x401F => {
                println!("disabled normally");
                0
            }
            ROM_START...ROM_END => {
                let mapper = self.mapper.borrow();
                mapper.ld_prg(address)
            }
        }
    }

    pub fn ld16(&mut self, address: u16) -> u16 {
        let l_byte = self.ld8(address);
        let r_byte = self.ld8(address + 1);
        (r_byte as u16) << 8 | (l_byte as u16)
    }
}
