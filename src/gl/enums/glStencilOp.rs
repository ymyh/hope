#[derive(Clone, Copy, PartialEq)]
pub enum GLStencilOp
{
    Keep,
    Zero,
    Replace,
    Increase,
    IncreaseWrap,
    Decrease,
    DecreaseWrap,
    Invert,
}