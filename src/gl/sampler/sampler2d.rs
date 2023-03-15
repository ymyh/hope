use std::sync::Arc;

use glam::Vec2;

use crate::{gl::util::{find_max16_power2, log2, sqrt, inv_sqrt, find_log2_max16}, gl::{glTexture::GLTexture, glColor::GLColor}};

use super::{WrapMode, GLFilterFunc, wrap, bilerp, isotropic_min_filter, sampler::Sampler, anisotropic_min_filter};

#[derive(Default, Clone)]
pub struct Sampler2D
{
    texture: Option<Arc<GLTexture>>,

    sampled_pixels_st: [Vec2; 3],
    sampled_pixels_uv: [Vec2; 4],
    sampled_idx: usize,

    wrap_s: WrapMode,
    wrap_t: WrapMode,

    min_filter: GLFilterFunc,
    mag_filter: GLFilterFunc,

    ddx: Vec2,
    ddy: Vec2,
    ddxy: Vec2,

    long_level: f32,
    aniso_level: f32,

    sample_point: i32,
}

impl From<Arc<GLTexture>> for Sampler2D
{
    fn from(texture: Arc<GLTexture>) -> Self
    {
        Self {
            texture: Some(texture),

            min_filter: GLFilterFunc::NearestMipmapNearest,
            mag_filter: GLFilterFunc::Nearest,

            sample_point: 1,

            ..Default::default()
        }
    }
}

impl Sampler for Sampler2D
{
    fn get_min_filter(&self) -> GLFilterFunc
    {
        self.min_filter
    }

    fn get_long_level(&self) -> f32
    {
        self.long_level
    }

    fn get_aniso_level(&self) -> f32
    {
        self.aniso_level
    }

    fn get_sample_point(&self) -> i32
    {
        self.sample_point
    }

    fn get_ddxy(&self) -> Vec2
    {
        self.ddxy
    }

    /// 这个函数应该由宏生成的函数自动调用，计算mipmap等级
    fn compute_level(&mut self, sample_point: i32)
    {
        self.ddx = self.sampled_pixels_st[1] - self.sampled_pixels_st[0];
        self.ddy = self.sampled_pixels_st[2] - self.sampled_pixels_st[0];

        let ddx_len_power2 = self.ddx.dot(self.ddx);
        let ddy_len_power2 = self.ddy.dot(self.ddy);

        self.long_level = log2(f32::max(ddx_len_power2, ddy_len_power2)) * 0.5;

        if sample_point != 1 && self.long_level > 0.
        {
            self.anisotropic_level(ddx_len_power2, ddy_len_power2, sample_point);
        }
        self.sample_point = sample_point;
    }
}

impl Sampler2D
{
    pub fn new(texture: Arc<GLTexture>) -> Self
    {
        Self::from(texture)
    }

    pub fn set_min_filter(&mut self, func: GLFilterFunc)
    {
        if func == GLFilterFunc::Linear || func == GLFilterFunc::Nearest
        {
            eprintln!("无效的缩小过滤器");
        }
        else
        {
            self.min_filter = func;   
        }
    }

    pub fn set_mag_filter(&mut self, func: GLFilterFunc)
    {
        if func == GLFilterFunc::Linear || func == GLFilterFunc::Nearest
        {
            self.mag_filter = func;
        }
        else
        {
            eprintln!("无效的放大过滤器");
        }
    }

    pub fn set_wrap_s(&mut self, mode: WrapMode)
    {
        self.wrap_s = mode;
    }

    pub fn set_wrap_t(&mut self, mode: WrapMode)
    {
        self.wrap_t = mode;
    }

    /// 采样纹理，记录st和uv坐标，声明一下，uv在代码里面全部是指映射前的值，范围是\[0, 1\]，st是指映射后的值
    pub fn sample(&mut self, uv: Vec2)
    {
        if self.sampled_idx < 3
        {
            let texture = self.texture.as_ref().unwrap();
            self.sampled_pixels_st[self.sampled_idx] = texture.compute_st(self.wrap(uv));
        }

        self.sampled_pixels_uv[self.sampled_idx] = uv;
        self.sampled_idx = if self.sampled_idx == 3 { 0 } else { self.sampled_idx + 1 };
    }

    /// 计算各项异性过滤mipmap等级
    #[inline]
    fn anisotropic_level(&mut self, ddx_len_power2: f32, ddy_len_power2: f32, sample_point: i32)
    {
        if ddx_len_power2 > ddy_len_power2
        {
            self.sample_point = i32::min(sample_point, find_max16_power2(
                i32::min((sqrt(ddx_len_power2) * inv_sqrt(ddy_len_power2)) as i32 + 1, 16)));

            self.ddxy = self.ddx;
            // self.aniso_level = log2(ddy_len_power2) * 0.5;
        }
        else 
        {
            self.sample_point = i32::min(sample_point, find_max16_power2(
                i32::min((sqrt(ddy_len_power2) * inv_sqrt(ddx_len_power2)) as i32 + 1, 16)));

            self.ddxy = self.ddy;
            // self.aniso_level = log2(ddx_len_power2) * 0.5;
        }

        self.aniso_level = self.long_level - find_log2_max16(self.sample_point) as f32;
    }

    ///获取颜色，根据采样器的设置进行过滤
    pub fn get_color(&mut self) -> GLColor
    {
        let uv = self.sampled_pixels_uv[self.sampled_idx];
        let texture = self.texture.as_ref().unwrap();

        let color;

        if self.long_level <= 0.
        {
            match self.mag_filter
            {
                GLFilterFunc::Nearest =>
                {
                    color = texture.get_value(texture.compute_st(uv) + 0.5)
                }

                GLFilterFunc::Linear =>
                {
                    color = bilerp(texture.compute_st(uv), texture)
                }

                _ => unreachable!()
            }
        }
        else
        {
            if self.sample_point == 1
            {
                color = isotropic_min_filter(self, uv, texture)
            }
            else
            {
                color = anisotropic_min_filter(self, uv, texture)
            }
        }

        self.sampled_idx = if self.sampled_idx == 3 { 0 } else { self.sampled_idx + 1 };
        color
    }

    fn wrap(&self, mut uv: Vec2) -> Vec2
    {
        uv.x = wrap(uv.x, self.wrap_s);
        uv.y = wrap(uv.y, self.wrap_t);

        uv
    }
}