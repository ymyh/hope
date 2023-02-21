use glam::Vec2;

use crate::gl::util::find_log2_max16;

use self::sampler::Sampler;

use super::{glTexture::GLTexture, glColor::GLColor};

pub mod sampler2d;
pub mod sampler;
pub mod cube_sampler;

#[derive(Clone, Copy, PartialEq, Default)]
pub enum WrapMode
{
    #[default]
    ClampToEdge, Repeat, MirroredRepeat
}

#[derive(Clone, Copy, PartialEq, Default)]
pub enum GLFilterFunc
{
    #[default]
    /// 临近过滤，仅用于放大过滤器
    Nearest,
    /// 双线性插值，仅用于放大过滤器
    Linear,
    /// 在最近的mipmap层使用临近过滤，仅用于缩小过滤器
    NearestMipmapNearest,
    /// 在两层mipmap之间使用临近过滤后再线性插值，仅用于缩小过滤器
    NearestMipmapLinear,
    /// 在最近的mipmap使用双线性插值，仅用于缩小过滤器
    LinearMipmapNearest,
    /// 三线性插值，仅用于缩小过滤器
    LinearMipmapLinear
}

#[inline(always)]
fn wrap(num: f32, mode: WrapMode) -> f32
{
    return match mode
    {
        WrapMode::ClampToEdge =>
        {
            num.clamp(0., 1.)
        }

        WrapMode::Repeat =>
        {
            num - num.floor()
        }

        WrapMode::MirroredRepeat =>
        {
            let x = num as i32;
            if x == 0
            {
                num
            }
            else
            {
                if x % 2 == 0
                {
                    1. - (num - num.floor())
                }
                else
                {
                    num - num.floor()
                }
            } 
        }
    };
}

fn bilerp(st: Vec2, texture: &GLTexture) -> GLColor
{
    let diff = ((st - st.floor()) * 256.).as_uvec2();

    let xp1 = (st.x + 1.).min(texture.width as f32 - 1.);
    let yp1 = (st.y + 1.).min(texture.height as f32 - 1.);

    let side_st = Vec2::new(xp1, st.y);
    let up_st = Vec2::new(st.x, yp1);
    let diagonal_st = Vec2::new(xp1, yp1);

    let mut this: u32 = texture.get_value(st).into();
    let mut side: u32 = texture.get_value(side_st).into();
    let mut up: u32 = texture.get_value(up_st).into();
    let mut diagonal: u32 = texture.get_value(diagonal_st).into();

    let s3 = diff.x * diff.y;
    let s0 = u32::wrapping_add(u32::wrapping_sub(u32::wrapping_sub(256 * 256, diff.y << 8), diff.x << 8), s3); // (256 - diff.x) * (256 - diff.y);
    let s1 = (diff.x << 8) - s3; // diff.x * (256 - diff.y);
    let s2 = (diff.y << 8) - s3; // diff.y * (256 - diff.x);

    let mut r = (this & 0x000000ff) * s0 + (side & 0x000000ff) * s1 +
    (up & 0x000000ff) * s2 + (diagonal & 0x000000ff) * s3;

    let mut f = (this & 0x0000ff00) * s0 + (side & 0x0000ff00) * s1 +
    (up & 0x0000ff00) * s2 + (diagonal & 0x0000ff00) * s3;

    r |= f & 0xff000000;

    this >>= 16;
    up >>= 16;
    side >>= 16;
    diagonal >>= 16;
    r >>= 16;

    f = (this & 0x000000ff) * s0 + (side & 0x000000ff) * s1 +
    (up & 0x000000ff) * s2 + (diagonal & 0x000000ff) * s3;

    r |= f & 0x00ff0000;

    f = (this & 0x0000ff00) * s0 + (side & 0x0000ff00) * s1 +
    (up & 0x0000ff00) * s2 + (diagonal & 0x0000ff00) * s3;

    r |= f & 0xff000000;

    r.into()
}

fn isotropic_min_filter<S: Sampler>(sampler: &S, uv: Vec2, texture: &GLTexture) -> GLColor
{
    match sampler.get_min_filter()
    {
        GLFilterFunc::NearestMipmapNearest =>
        {
            let tex = texture.get_mipmap(sampler.get_long_level().round());
            tex.get_value(tex.compute_st(uv) + 0.5)
        }

        GLFilterFunc::LinearMipmapNearest =>
        {
            let tex = texture.get_mipmap(sampler.get_long_level().round());
            bilerp(tex.compute_st(uv), tex)
        }

        GLFilterFunc::NearestMipmapLinear =>
        {
            let left  = texture.get_mipmap(sampler.get_long_level().floor());
            let right = texture.get_mipmap(sampler.get_long_level().ceil());

            let st_left  = left.compute_st(uv);
            let st_right = right.compute_st(uv);

            left.get_value(st_left).lerp(right.get_value(st_right), sampler.get_long_level() - sampler.get_long_level().floor())
        }

        GLFilterFunc::LinearMipmapLinear =>
        {
            let left  = texture.get_mipmap(sampler.get_long_level().floor());
            let right = texture.get_mipmap(sampler.get_long_level().ceil());

            let a = bilerp(left .compute_st(uv), left);
            let b = bilerp(right.compute_st(uv), right);

            a.lerp(b, sampler.get_long_level() - sampler.get_long_level().floor())
        }

        _ => unreachable!()
    }
}

fn anisotropic_min_filter<S: Sampler>(sampler: &S, uv: Vec2, texture: &GLTexture) -> GLColor
{
    let mut r = 0u32;
    let mut g = 0u32;
    let mut b = 0u32;
    let mut a = 0u32;

    match sampler.get_min_filter()
    {
        GLFilterFunc::NearestMipmapNearest =>
        {
            let tex = texture.get_mipmap(sampler.get_aniso_level().round());
            let footprint_center = tex.compute_st(uv);

            for i in 0..sampler.get_sample_point()
            {
                let c = tex.get_value(footprint_center + sampler.get_ddxy() * get_offset(sampler.get_sample_point(), i));

                r += c.r as u32;
                g += c.g as u32;
                b += c.b as u32;
                a += c.a as u32;
            }
        },

        GLFilterFunc::NearestMipmapLinear =>
        {
            let tex = texture.get_mipmap(sampler.get_aniso_level().round());
            let footprint_center = tex.compute_st(uv);

            for i in 0..sampler.get_sample_point()
            {
                let c = bilerp(footprint_center + sampler.get_ddxy() * get_offset(sampler.get_sample_point(), i), tex);

                r += c.r as u32;
                g += c.g as u32;
                b += c.b as u32;
                a += c.a as u32;
            }
        }

        GLFilterFunc::LinearMipmapNearest =>
        {
            let left  = texture.get_mipmap(sampler.get_aniso_level().floor());
            let right = texture.get_mipmap(sampler.get_aniso_level().ceil());

            let footprint_center_left = left.compute_st(uv);
            let footprint_center_right = right.compute_st(uv);

            for i in 0..sampler.get_sample_point()
            {
                let c_left = left.get_value(footprint_center_left + sampler.get_ddxy() * get_offset(sampler.get_sample_point(), i));
                let c_right = right.get_value(footprint_center_right + sampler.get_ddxy() * get_offset(sampler.get_sample_point(), i));

                let c = c_left.lerp(c_right, sampler.get_aniso_level() - sampler.get_aniso_level().floor());

                r += c.r as u32;
                g += c.g as u32;
                b += c.b as u32;
                a += c.a as u32;
            }
        }

        GLFilterFunc::LinearMipmapLinear =>
        {
            let left  = texture.get_mipmap(sampler.get_aniso_level().floor());
            let right = texture.get_mipmap(sampler.get_aniso_level().ceil());

            let footprint_center_left = left.compute_st(uv);
            let footprint_center_right = right.compute_st(uv);

            for i in 0..sampler.get_sample_point()
            {
                let st_left = footprint_center_left + sampler.get_ddxy() * get_offset(sampler.get_sample_point(), i);
                let st_right = footprint_center_right + sampler.get_ddxy() * get_offset(sampler.get_sample_point(), i);

                let c_left  = bilerp(st_left, left);
                let c_right = bilerp(st_right, right);

                let c = c_left.lerp(c_right, sampler.get_aniso_level() - sampler.get_aniso_level().floor());

                r += c.r as u32;
                g += c.g as u32;
                b += c.b as u32;
                a += c.a as u32;
            }
        }

        _ => unreachable!(),
    }

    let div = find_log2_max16(sampler.get_sample_point()) as u32;
    r >>= div;
    g >>= div;
    b >>= div;
    a >>= div;

    (a << 24 | b << 16 | g << 8 | r).into()
}

fn get_offset(sample_point: i32, i: i32) -> f32
{
    const LUT: [f32; 32] = [
        0.,
        0.,

        1. / (2. + 1.) - 0.5,
        2. / (2. + 1.) - 0.5,

        1. / (4. + 1.) - 0.5,
        2. / (4. + 1.) - 0.5,
        3. / (4. + 1.) - 0.5,
        4. / (4. + 1.) - 0.5,

        1. / (8. + 1.) - 0.5,
        2. / (8. + 1.) - 0.5,
        3. / (8. + 1.) - 0.5,
        4. / (8. + 1.) - 0.5,
        5. / (8. + 1.) - 0.5,
        6. / (8. + 1.) - 0.5,
        7. / (8. + 1.) - 0.5,
        8. / (8. + 1.) - 0.5,

        1. / (16. + 1.) - 0.5,
        2. / (16. + 1.) - 0.5,
        3. / (16. + 1.) - 0.5,
        4. / (16. + 1.) - 0.5,
        5. / (16. + 1.) - 0.5,
        6. / (16. + 1.) - 0.5,
        7. / (16. + 1.) - 0.5,
        8. / (16. + 1.) - 0.5,
        9. / (16. + 1.) - 0.5,
        10. / (16. + 1.) - 0.5,
        11. / (16. + 1.) - 0.5,
        12. / (16. + 1.) - 0.5,
        13. / (16. + 1.) - 0.5,
        14. / (16. + 1.) - 0.5,
        15. / (16. + 1.) - 0.5,
        16. / (16. + 1.) - 0.5,
    ];

    LUT[(sample_point + i) as usize]
}