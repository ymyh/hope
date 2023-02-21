#![allow(non_camel_case_types)]

use std::sync::Arc;

use glam::{Vec3, Vec2};

use crate::{gl::{glTexture::GLTexture, glColor::GLColor}, gl::{util::{log2, find_max16_power2, sqrt, inv_sqrt, find_log2_max16}}};

use super::{GLFilterFunc, sampler::Sampler, bilerp, isotropic_min_filter, anisotropic_min_filter};

#[derive(Default, PartialEq, Clone, Copy)]
enum Face
{
    #[default]
    POS_X,
    NEG_X,

    POS_Y,
    NEG_Y,

    POS_Z,
    NEG_Z,
}

/// 概念基本同Sampler2D
#[derive(Default, Clone)]
pub struct CubeSampler
{
    textures: Option<[Arc<GLTexture>; 6]>,

    sampled_pixels_st: [Vec2; 3],
    sampled_pixels_uv: [Vec2; 4],
    sampled_pixels_face: [Face; 4],
    sampled_idx: usize,

    ddx: Vec2,
    ddy: Vec2,
    ddxy: Vec2,

    long_level: f32,
    aniso_level: f32,

    sample_point: i32,

    min_filter: GLFilterFunc,
    mag_filter: GLFilterFunc,
}

impl Sampler for CubeSampler
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
}

impl CubeSampler
{
    pub fn from_textures(pos_x: Arc<GLTexture>, neg_x: Arc<GLTexture>,
    pos_y: Arc<GLTexture>, neg_y: Arc<GLTexture>,
    pos_z: Arc<GLTexture>, neg_z: Arc<GLTexture>) -> Option<Self>
    {
        if (pos_x.width != neg_x.width ||
        pos_x.width != pos_y.width ||
        pos_x.width != neg_y.width ||
        pos_x.width != pos_z.width ||
        pos_x.width != neg_z.width)
        ||
        (pos_x.height != neg_x.height ||
        pos_x.height != pos_y.height ||
        pos_x.height != neg_y.height ||
        pos_x.height != pos_z.height ||
        pos_x.height != neg_z.height)
        || pos_x.width != pos_x.height
        {
            return None;
        }

        Some(Self {
            textures: Some([pos_x, neg_x, pos_y, neg_y, pos_z, neg_z]),

            min_filter: GLFilterFunc::NearestMipmapNearest,
            mag_filter: GLFilterFunc::Nearest,

            sample_point: 1,

            ..Default::default()
        })
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

    pub fn sample(&mut self, uvw: Vec3)
    {
        let uvw_abs = uvw.abs();
        let face;
        let uv;

        let max = f32::max(f32::max(uvw_abs.x, uvw_abs.y), uvw_abs.z);

        if max == uvw_abs.x
        {
            if uvw.x > 0.
            {
                face = Face::POS_X;
                uv = (Vec2::new(-uvw.z, -uvw.y) / uvw_abs.x + 1.) * 0.5;
            }
            else
            {
                face = Face::NEG_X;
                uv = (Vec2::new(uvw.z, -uvw.y) / uvw_abs.x + 1.) * 0.5;
            }
        }
        else if max == uvw_abs.y
        {
            if uvw.y > 0.
            {
                face = Face::POS_Y;
                uv = (Vec2::new(-uvw.x, -uvw.z) / uvw_abs.y + 1.) * 0.5;
            }
            else
            {
                face = Face::NEG_Y;
                uv = (Vec2::new(-uvw.x, uvw.z) / uvw_abs.y + 1.) * 0.5;
            }
        }
        else
        {
            if uvw.z > 0.
            {
                face = Face::POS_Z;
                uv = (Vec2::new(uvw.x, -uvw.y) / uvw_abs.z + 1.) * 0.5;
            }
            else
            {
                face = Face::NEG_Z;
                uv = (Vec2::new(-uvw.x, -uvw.y) / uvw_abs.z + 1.) * 0.5;
            }
        }

        if self.sampled_idx < 3
        {
            let st = unsafe { self.textures.as_ref().unwrap_unchecked()[0].compute_st(uv) };
            self.sampled_pixels_st[self.sampled_idx] = Vec2::new(st.x, st.y);
        }

        self.sampled_pixels_uv[self.sampled_idx] = uv;
        self.sampled_pixels_face[self.sampled_idx] = face;

        self.sampled_idx = if self.sampled_idx == 3 { 0 } else { self.sampled_idx + 1 };
    }

    pub fn compute_level(&mut self, sample_point: i32)
    {
        if self.sampled_pixels_face[0] == self.sampled_pixels_face[1] && self.sampled_pixels_face[1] == self.sampled_pixels_face[2]
        {
            self.ddx = self.sampled_pixels_st[1] - self.sampled_pixels_st[0];
            self.ddy = self.sampled_pixels_st[2] - self.sampled_pixels_st[0];
    
            let ddx_len_power2 = self.ddx.dot(self.ddx);
            let ddy_len_power2 = self.ddy.dot(self.ddy);
    
            self.long_level = log2(f32::max(ddx_len_power2, ddy_len_power2)) * 0.5;
    
            if sample_point != 1 && self.long_level > 0.
            {
                self.anisotropic_level(ddx_len_power2, ddy_len_power2);
            }
    
            self.sample_point = sample_point;
        }
        else
        {
            self.long_level = 0.;
            self.sample_point = 1;
        }
    }

    #[inline]
    pub fn anisotropic_level(&mut self, ddx_len_power2: f32, ddy_len_power2: f32)
    {
        if ddx_len_power2 > ddy_len_power2
        {
            self.sample_point = i32::min(self.sample_point, find_max16_power2(
                i32::min((sqrt(ddx_len_power2) * inv_sqrt(ddy_len_power2)) as i32, 16)));

            self.ddxy = self.ddx;
            // self.aniso_level = log2(ddy_len_power2) * 0.5;
        }
        else 
        {
            self.sample_point = i32::min(self.sample_point, find_max16_power2(
                i32::min((sqrt(ddy_len_power2) * inv_sqrt(ddx_len_power2)) as i32, 16)));

            self.ddxy = self.ddy;
            // self.aniso_level = log2(ddx_len_power2) * 0.5;
        }

        self.aniso_level = self.long_level - find_log2_max16(self.sample_point) as f32;
    }

    pub fn get_color(&mut self) -> GLColor
    {
        let texture = &self.textures.as_ref().unwrap()[self.sampled_pixels_face[self.sampled_idx] as usize];
        let uv = self.sampled_pixels_uv[self.sampled_idx];

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
}