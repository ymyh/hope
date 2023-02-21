#[repr(i32)]
#[derive(Clone, Copy, Default, PartialEq)]
pub enum GLSamplePoint
{
    #[default]
    X1 = 1, X2 = 2, X4 = 4, X8 = 8, X16 = 16
}