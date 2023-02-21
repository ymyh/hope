#[derive(Clone, Copy, PartialEq)]
pub enum GLBlendFunc
{
    Zero,
    One,

    SrcAlpha,
    DstAlpha,
    OneMinusSrcAlpha,
    OneMinusDstAlpha,

    SrcColor,
    DstColor,
    OneMinusSrcColor,
    OneMinusDstColor, 
}