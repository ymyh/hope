use std::{fs::File, io::BufReader, f32::consts::PI};

use glam::{Mat4, Vec4, Vec3, Vec3A};
use image::ImageFormat;

pub struct Resolution
{
    pub width: u32,
    pub height: u32,
}

impl Resolution
{
    pub fn ratio(&self) -> f32
    {
        self.width as f32 / self.height as f32
    }

    pub fn pitch(&self) -> usize
    {
        self.width as usize * 4
    }

    pub fn width_f(&self) -> f32
    {
        self.width as f32
    }

    pub fn height_f(&self) -> f32
    {
        self.height as f32
    }
}

/// 为 C-like 枚举实现 `From<i32>`, `BitOr`, `BitOrAssign`, `BitAnd`, `BitAndAssign`, `BitXor`, `BitXorAssign`，
/// 且并不保证枚举值的正确性。
#[macro_export]
macro_rules! c_enum_impl
{
    ($t:ty) =>
    {
        impl From<i32> for $t
        {
            fn from(value: i32) -> Self
            {
                unsafe { std::mem::transmute(value) }
            }
        }

        impl std::ops::BitOr for $t
        {
            type Output = Self;

            fn bitor(self, rhs: Self) -> Self::Output
            {
                (self as i32 | rhs as i32).into()
            }
        }

        impl std::ops::BitOrAssign for $t
        {
            fn bitor_assign(&mut self, rhs: Self)
            {
                *self = (*self as i32 | rhs as i32).into();
            }
        }

        impl std::ops::BitAnd for $t
        {
            type Output = Self;

            fn bitand(self, rhs: Self) -> Self::Output
            {
                (self as i32 & rhs as i32).into()
            }
        }

        impl std::ops::BitAndAssign for $t
        {
            fn bitand_assign(&mut self, rhs: Self)
            {
                *self = (*self as i32 & rhs as i32).into();
            }
        }

        impl std::ops::BitXor for $t
        {
            type Output = Self;

            fn bitxor(self, rhs: Self) -> Self::Output
            {
                (self as i32 ^ rhs as i32).into()
            }
        }

        impl std::ops::BitXorAssign for $t
        {
            fn bitxor_assign(&mut self, rhs: Self)
            {
                *self = (*self as i32 ^ rhs as i32).into();
            }
        }
    };
}

#[inline]
pub fn ortho_projection(left: f32, right: f32, bottom: f32, top: f32, near: f32, far: f32) -> Mat4
{
    Mat4 {
        x_axis: Vec4::new(2. / (right - left), 0., 0., 0.),
        y_axis: Vec4::new(0., 2. / (top - bottom), 0., 0.),
        z_axis: Vec4::new(0., 0., 2. / (far - near), 0.),
        w_axis: Vec4::new((left + right) / (left - right),
            (bottom + top) / (bottom - top),
            (near + far) / (near - far),
            1.),
    }
}

#[inline]
pub fn perspective_projection(fov: f32, aspect: f32, near: f32, far: f32) -> Mat4
{
    let f = f32::tan(PI * 0.5 - 0.5 * fov);
    let inv_range = 1. / (near - far);

    Mat4 {
        x_axis: Vec4::new(f / aspect, 0., 0., 0.),
        y_axis: Vec4::new(0., f, 0., 0.),
        z_axis: Vec4::new(0., 0., (near + far) * inv_range, -1.),
        w_axis: Vec4::new(0., 0., near * far * inv_range * 2., 0.)
    }
}

#[inline]
pub fn frustum(left: f32, right: f32, bottom: f32, top: f32, near: f32, far: f32) -> Mat4
{
    let dx = 1. / (right - left);
    let dy = 1. / (top - bottom);
    let dz = 1. / (far - near);

    Mat4 {
        x_axis: Vec4::new(2. * near * dx, 0., 0., 0.),
        y_axis: Vec4::new(0., 2. * near * dy, 0., 0.),
        z_axis: Vec4::new((left + right) * dx, (top + bottom) * dy, -(far + near) * dz, -1.),
        w_axis: Vec4::new(0., 0., -2. * near * far * dz, 0.)
    }
}

pub fn reflect(dir: Vec3, norm: Vec3) -> Vec3
{
    dir - (dir.dot(norm) * 2.) * norm
}

pub fn reflect_simd(dir: Vec3A, norm: Vec3A) -> Vec3A
{
    dir - (dir.dot(norm) * 2.) * norm
}

#[inline]
pub fn look_at(pos: Vec3, target: Vec3, up: Vec3) -> Mat4
{
    let z_axis = (pos - target).normalize();
    let x_axis = up.cross(z_axis).normalize();
    let y_axis = z_axis.cross(x_axis).normalize();
    
    Mat4 {
        x_axis: Vec4::new(x_axis.x, x_axis.y, x_axis.z, 0.),
        y_axis: Vec4::new(y_axis.x, y_axis.y, y_axis.z, 0.),
        z_axis: Vec4::new(z_axis.x, z_axis.y, z_axis.z, 0.),
        w_axis: Vec4::new(pos.x, pos.y, pos.z, 1.), 
    }
}

pub fn read_image(path: &str) -> Vec<u8>
{
    let f = File::open(path).unwrap();
    let mut format = None;

    if let Some((_, suffix)) = path.rsplit_once(".")
    {
        format = match suffix
        {
            "png" => Some(ImageFormat::Png),
            "jpeg" | "jpg" => Some(ImageFormat::Jpeg),
            "bmp" => Some(ImageFormat::Bmp),
            "gif" => Some(ImageFormat::Gif),
            "tga" => Some(ImageFormat::Tga),
            "ico" => Some(ImageFormat::Ico),
            "tif" | "tiff" => Some(ImageFormat::Tiff),

            _ => None,
        };
    }

    let img = image::load(BufReader::new(f), format.unwrap()).unwrap();
    let result: Vec<u8> = img.into_rgba8().to_vec();

    result
}

#[inline(always)]
pub fn is_between<T: Ord>(value: T, min: T, max: T) -> bool
{
    value >= min && value <= max
}

/// 非常经典的一段代码
pub fn inv_sqrt(mut num: f32) -> f32
{
	let mut i: u32;
    let half_num = num * 0.5;

	i  = unsafe { std::mem::transmute(num) };
	i  = 0x5f375a86 - (i >> 1);
	num  = unsafe { std::mem::transmute(i) };
	num  = num * ( 1.5 - ( half_num * num * num ) );
	// num  = num * ( 1.5 - ( x2 * num * num ) );

	num
}

/// 同样很经典
pub fn sqrt(mut num: f32) -> f32
{
    let a = num;
    let mut i: u32 = unsafe { std::mem::transmute(num) };

    i = (i + 0x3f76cf62) >> 1;
    num = unsafe { std::mem::transmute(i) };
    num = (num + a / num) * 0.5;
    // num = (num + a / num) * 0.5;

    num
}

pub fn log2(num: f32) -> f32
{
    let i: u32 = unsafe { std::mem::transmute(num) };
    i as f32 * (1. / 8388608.0) - (127.0 - 0.0450466)
}

pub fn div_255(num: u16) -> u16
{
    (num + 1 + ((num + 1) >> 8)) >> 8
}

/// 查找小于等于16某数的最大二次幂
pub const fn find_max16_power2(num: i32) -> i32
{
    const LUT: [i32; 17] = [1, 1, 2, 2, 4, 4, 4, 4, 8, 8, 8, 8, 8, 8, 8, 8, 16];
    LUT[num as usize]
}

/// 查找从0到16的log2值
pub const fn find_log2_max16(num: i32) -> f32
{
    const LUT: [f32; 17] = [f32::NEG_INFINITY, 0., 1., 1.5849625, 2., 2.321928, 2.5849624, 2.807355,
    3., 3.169925, 3.321928, 3.4594316, 3.5849624, 3.7004397, 3.807355, 3.9068906, 4.];
    LUT[num as usize]
}