use std::ops::{Mul, Add};

/// 这个trait应该由同名derive宏自动实现
pub trait Varying : Add<Output = Self> +
    Mul<f32, Output = Self> +
    Clone + Copy + Default + Send + Sync
{

}