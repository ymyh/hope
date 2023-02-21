use crate::c_enum_impl;

#[repr(i32)]
#[derive(Clone, Copy, Default, PartialEq)]
pub enum GLBufferBit
{
    #[default]
    Color   =  0b001,
    Depth   =  0b010,
    Stencil =  0b100
}

c_enum_impl!(GLBufferBit);