use std::ops::{Add, Mul, AddAssign, MulAssign, Sub, SubAssign};

use glam::Vec4;

use crate::gl::util::div_255;

#[macro_export]
macro_rules! make_color
{
    ($r: expr, $g: expr, $b: expr, $a: expr) =>
    {
        GLColor::new($r as u8, $g as u8, $b as u8, $a as u8)
    };

    ($r: expr, $g: expr, $b: expr) =>
    {
        GLColor::new($r as u8, $g as u8, $b as u8, 255)
    };

    ($c: expr) =>
    {
        GLColor::new($c as u8, $c as u8, $c as u8, $c as u8)
    };

    ($c: expr, $a: expr) =>
    {
        GLColor::new($c as u8, $c as u8, $c as u8, $a as u8)
    }
}

#[derive(Clone, Copy, Default, Debug)]
pub struct GLColor
{
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl GLColor
{
    pub const ONE: GLColor = GLColor::new(255, 255, 255, 255);
    pub const ZERO: GLColor = GLColor::new(0, 0, 0, 0);

    pub const fn new(r: u8, g: u8, b: u8, a: u8) -> Self
    {
        return GLColor { r, g, b, a };
    }

    pub fn lerp(self, other: Self, p: f32) -> Self
    {
        let p = (p * 2048.) as u32;
        let one_minus = 2048 - p;

        let r = (self.r as u32 * one_minus + other.r as u32 * p) >> 11;
        let g = (self.g as u32 * one_minus + other.g as u32 * p) >> 11;
        let b = (self.b as u32 * one_minus + other.b as u32 * p) >> 11;
        let a = (self.a as u32 * one_minus + other.a as u32 * p) >> 11;

        ((a << 24 | b << 16 | g << 8 | r) as u32).into()
    }

    pub fn max(&self, rhs: Self) -> Self
    {
        GLColor {
            r: self.r.max(rhs.r),
            g: self.g.max(rhs.g),
            b: self.b.max(rhs.b),
            a: self.a.max(rhs.a),
        }
    }

    pub fn min(&self, rhs: Self) -> Self
    {
        GLColor {
            r: self.r.min(rhs.r),
            g: self.g.min(rhs.g),
            b: self.b.min(rhs.b),
            a: self.a.min(rhs.a),
        }
    }

    pub fn from_str(color: &str) -> Option<GLColor>
    {
        if !color.starts_with("#")
        {
            return None;
        }

        let r;
        let g;
        let b;
        let a;

        match color.len()
        {
            7 =>
            {
                let sr = &color[1..3];
                let sg = &color[3..5];
                let sb = &color[5..7];

                r = u8::from_str_radix(sr, 16).unwrap();
                g = u8::from_str_radix(sg, 16).unwrap();
                b = u8::from_str_radix(sb, 16).unwrap();

                a = 255;
            }

            9 =>
            {
                let sr = &color[1..3];
                let sg = &color[3..5];
                let sb = &color[5..7];
                let sa = &color[7..9];

                r = u8::from_str_radix(sr, 16).unwrap();
                g = u8::from_str_radix(sg, 16).unwrap();
                b = u8::from_str_radix(sb, 16).unwrap();
                a = u8::from_str_radix(sa, 16).unwrap();
            }

            _ =>
            {
                return None;
            }
        }

        Some(GLColor {
            r, g, b, a
        })

    }
}

impl From<Vec4> for GLColor
{
    fn from(vec: Vec4) -> Self
    {
        let vec = vec * 255.;
        Self {
            r: vec.x as u8,
            g: vec.y as u8,
            b: vec.z as u8,
            a: vec.w as u8
        }
    }
}

impl From<u32> for GLColor
{
    fn from(color: u32) -> Self
    {
        unsafe
        {
            std::mem::transmute(color)
        }
    }
}

impl Into<Vec4> for GLColor
{
    fn into(self) -> Vec4
    {
        Vec4::new(self.r as f32,
            self.g as f32,
            self.b as f32,
            self.a as f32) * (1. / 255.)
    }
}

impl Into<u32> for GLColor
{
    fn into(self) -> u32
    {
        unsafe
        {
            std::mem::transmute(self)
        }
    }
}

impl Into<(u16, u16, u16, u16)> for GLColor
{
    fn into(self) -> (u16, u16, u16, u16)
    {
        (self.r as u16, self.g as u16, self.b as u16, self.a as u16)
    }
}

impl Add<Self> for GLColor
{
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output
    {
        Self {
            r: self.r.saturating_add(rhs.r),
            g: self.g.saturating_add(rhs.g),
            b: self.b.saturating_add(rhs.b),
            a: self.a.saturating_add(rhs.a),
        }
    }
}

impl AddAssign<Self> for GLColor
{
    fn add_assign(&mut self, rhs: Self)
    {
        self.r = self.r.saturating_add(rhs.r);
        self.g = self.g.saturating_add(rhs.g);
        self.b = self.b.saturating_add(rhs.b);
        self.a = self.a.saturating_add(rhs.a);
    }
}

impl Sub<Self> for GLColor
{
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output
    {
        Self {
            r: self.r.saturating_sub(rhs.r),
            g: self.g.saturating_sub(rhs.g),
            b: self.b.saturating_sub(rhs.b),
            a: self.a.saturating_sub(rhs.a),
        }
    }
}

impl SubAssign for GLColor
{
    fn sub_assign(&mut self, rhs: Self)
    {
        self.r = self.r.saturating_sub(rhs.r);
        self.g = self.g.saturating_sub(rhs.g);
        self.b = self.b.saturating_sub(rhs.b);
        self.a = self.a.saturating_sub(rhs.a);
    }
}

impl Mul<Self> for GLColor
{
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output
    {
        let (lr, lg, lb, la) = self.into();
        let (rr, rg, rb, ra) = rhs.into();

        make_color! {
            div_255(lr * rr),
            div_255(lg * rg),
            div_255(lb * rb),
            div_255(la * ra)
        }
    }
}

impl MulAssign<Self> for GLColor
{
    fn mul_assign(&mut self, rhs: Self)
    {
        let (lr, lb, lg, la) = (*self).into();
        let (rr, rb, rg, ra) = rhs.into();

        self.r = div_255(lr * rr) as u8;
        self.b = div_255(lg * rg) as u8;
        self.g = div_255(lb * rb) as u8;
        self.a = div_255(la * ra) as u8;
    }
}

impl Mul<u8> for GLColor
{
    type Output = Self;

    fn mul(self, rhs: u8) -> Self::Output
    {
        let rhs = rhs as u16;
        let (r, g, b, a) = self.into();
        make_color!(div_255(r * rhs), div_255(g * rhs), div_255(b * rhs), div_255(a * rhs))
    }
}

impl Mul<f32> for GLColor
{
    type Output = Self;

    fn mul(self, rhs: f32) -> Self::Output
    {
        let ratio = (rhs * 2048.) as i32;

        let r = (self.r as i32 * ratio) >> 11;
        let g = (self.g as i32 * ratio) >> 11;
        let b = (self.b as i32 * ratio) >> 11;
        let a = (self.a as i32 * ratio) >> 11;

        ((a << 24 | b << 16 | g << 8 | r) as u32).into()
    }
}