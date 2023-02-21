use glam::{Vec4, IVec2};

use crate::gl::glColor::GLColor;

use super::varying::Varying;

pub trait Program<V: Varying>
{
    fn vertex(&mut self, index: usize) -> Vec4;
    fn fragment(&mut self, varying: &V, pos: IVec2) -> GLColor;

    fn sample(&mut self, varying: &V);
}