use glam::Vec2;

use super::GLFilterFunc;

pub trait Sampler
{
    fn get_min_filter(&self) -> GLFilterFunc;
    fn get_long_level(&self) -> f32;
    fn get_aniso_level(&self) -> f32;
    fn get_sample_point(&self) -> i32;
    fn get_ddxy(&self) -> Vec2;

    fn compute_level(&mut self, sample_point: i32);
}