#![allow(non_camel_case_types)]

#[repr(i32)]
#[derive(Clone, Copy, PartialEq)]
pub enum GLFunction
{
    AlphaTest,

    Blend,

    CullFace,

    DepthTest,
    StencilTest,

    Reciprocal_W,
    Z,
}