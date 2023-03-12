use crate::c_enum_impl;

#[repr(i32)]
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum GLCompareFunc
{
    Never     = 0b000,
    Equal     = 0b001,
    Less      = 0b010,
    EqLess    = 0b011,
    Greater   = 0b100,
    EqGreater = 0b101,
    NotEqual  = 0b110,
    Always    = 0b111,
}

c_enum_impl!(GLCompareFunc);