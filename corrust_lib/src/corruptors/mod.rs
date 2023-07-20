use std::fmt::{self, Display};

use rand::{random, thread_rng, Rng};
pub struct Input {
    pub data: Vec<u8>,
    pub start_offset: usize,
    pub end_offset: usize,
}

pub trait Corruptor {
    fn corrupt(&self, input: &Input) -> Vec<u8>;
}

pub struct RandCorruptor {
    pub intensity: i32,
}

impl Corruptor for RandCorruptor {
    fn corrupt(&self, input: &Input) -> Vec<u8> {
        let mut rng = thread_rng();
        let mut data = input.data.clone();
        for i in 0..self.intensity {
            let off = rng.gen_range(input.start_offset..input.end_offset);
            data[off] = rng.gen_range(0..=255);
        }
        data
    }
}

impl Default for RandCorruptor {
    fn default() -> RandCorruptor {
        RandCorruptor { intensity: 100 }
    }
}

pub struct TiltCorruptor {
    pub intensity: i32,
    pub tilt: i8,
}

impl Corruptor for TiltCorruptor {
    fn corrupt(&self, input: &Input) -> Vec<u8> {
        let mut rng = thread_rng();
        let mut data = input.data.clone();
        for _i in 0..self.intensity {
            let off = rng.gen_range(input.start_offset..input.end_offset);
            data[off] = match self.tilt.signum() {
                1 => data[off].wrapping_add(self.tilt.abs() as u8),
                -1 => data[off].wrapping_sub(self.tilt.abs() as u8),
                _ => data[off],
            }
        }
        data
    }
}

impl Default for TiltCorruptor {
    fn default() -> TiltCorruptor {
        TiltCorruptor {
            intensity: 100,
            tilt: 1,
        }
    }
}

#[derive(PartialEq)]
pub enum BitOp {
    AND,
    OR,
    NAND,
}

impl Display for BitOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BitOp::AND => write!(f, "AND"),
            BitOp::OR => write!(f, "OR"),
            BitOp::NAND => write!(f, "NAND"),
        }
    }
}

pub struct BitwiseCorruptor {
    pub intensity: i32,
    pub op: BitOp,
    pub rhs: u8,
}

impl Corruptor for BitwiseCorruptor {
    fn corrupt(&self, input: &Input) -> Vec<u8> {
        let mut rng = thread_rng();
        let mut data = input.data.clone();
        for i in 0..self.intensity {
            let off = rng.gen_range(input.start_offset..input.end_offset);
            data[off] = match self.op {
                BitOp::AND => data[off] & self.rhs,
                BitOp::OR => data[off] | self.rhs,
                BitOp::NAND => data[off] & !self.rhs,
                _ => data[off],
            }
        }
        data
    }
}

impl Default for BitwiseCorruptor {
    fn default() -> BitwiseCorruptor {
        BitwiseCorruptor {
            intensity: 100,
            op: BitOp::AND,
            rhs: 1,
        }
    }
}

pub fn get_corruptor<T: Default>() -> T {
    T::default()
}

pub mod chain;
