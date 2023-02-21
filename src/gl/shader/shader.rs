use super::varying::Varying;

/// 这个trait应该由同名derive宏自动实现
pub trait Shader<V: Varying> : Default
{
    fn next(&mut self);
    fn reset(&mut self);

    fn get_varying(&self) -> &Vec<V>;

    fn compute_level(&mut self, sample_point: i32);
}