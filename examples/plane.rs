use std::sync::Arc;

use glam::{Vec4, IVec2, Mat4, EulerRot, Vec3, Vec2};
use hope::gl::enums::glBufferBit::GLBufferBit;
use hope::gl::enums::glSamplePoint::GLSamplePoint;
use hope::gl::glColor::GLColor;
use hope::gl::glTexture::GLTexture;
use hope::gl::sampler::GLFilterFunc;
use hope::gl::sampler::sampler2d::Sampler2D;
use hope::gl::util::read_image;
use hope::gl::{glContext::GLContext, shader::program::Program};
use hope::gl::shader::shader::Shader;
use hope::make_color;
use image::ColorType;
use shader::Shader;
use varying::Varying;
use hope::gl::shader::varying::Varying;

#[derive(Default, Shader)]
struct TestShader
{
    #[varying(TextureVarying)]
    varyings: Vec<TextureVarying>,

    positions: Vec<Vec4>,
    tex_coords: Vec<Vec2>,

    #[uniform]
    mvp: Mat4,

    #[sampler]
    book: Sampler2D,
}

impl Program<TextureVarying> for TestShader
{
    fn vertex(&mut self, index: usize) -> Vec4
    {
        self.varyings.push(TextureVarying { tex_coords: self.tex_coords[index] });
        self.mvp * self.positions[index]
    }

    //片段着色器的后半部分，可以直接从纹理当中取颜色
    fn fragment(&mut self, _: &TextureVarying, _: IVec2) -> GLColor
    {
        self.book.get_color()
    }

    //是片段着色器的前半部分，使用sample函数为纹理采样
    fn sample(&mut self, varying: &TextureVarying)
    {
        self.book.sample(varying.tex_coords);
    }
}

#[derive(Clone, Copy, Default, Varying)]
struct TextureVarying
{
    tex_coords: Vec2,
}

fn main()
{
    let mut gl = GLContext::new(1920, 1080);
    let mut fb = gl.create_buffer();
    let mut shader = TestShader::default();

    //添加颜色附着
    fb.attach_color();

    let (vertices, tex_coords) = make_quad(1024.0, 512.0);
    shader.positions = vertices;
    shader.tex_coords = tex_coords;

    let mut book = GLTexture::from_bytes(&read_image("./img/book.png"), 1024, 512).unwrap();
    book.create_mipmap(None);

    shader.book = Sampler2D::new(Arc::new(book));
    shader.book.set_mag_filter(GLFilterFunc::Linear);
    shader.book.set_min_filter(GLFilterFunc::LinearMipmapLinear);
    
    let proj = Mat4::perspective_rh(60f32.to_radians(), 1920. / 1080., 1., 2000.);
    let view = Mat4::look_at_rh(Vec3::new(0., 0., -1.), Vec3::new(0., 0., 0.), Vec3::new(0., -1., 0.)).inverse();
    let model = Mat4::from_translation(Vec3::new(-100.0, -250.0, 200.0)) *
    Mat4::from_euler(EulerRot::ZXY, 0f32.to_radians(), 0f32.to_radians(), -45f32.to_radians());
    shader.mvp = proj * view * model;

    //各向异性采样数量
    gl.anisotropic_filter(GLSamplePoint::X1);

    gl.clear_color(make_color!(255));
    gl.clear(GLBufferBit::Color, &mut fb);

    gl.draw_arrays(&mut shader, 6, 0, &mut fb);

    image::save_buffer("./plane.png", fb.get_color_buffer(), 1920, 1080, ColorType::Rgba8).unwrap()
}

fn make_quad(width: f32, height: f32) -> (Vec<Vec4>, Vec<Vec2>)
{
    let vertices = vec![
        Vec4::new(0., 0., 0., 1.),
        Vec4::new(width, height ,0., 1.),
        Vec4::new(width, 0., 0., 1.),

        Vec4::new(0., 0., 0., 1.),
        Vec4::new(0., height, 0., 1.),
        Vec4::new(width, height ,0., 1.),
    ];

    let tex_coords = vec![
        Vec2::new(0.0, 0.0),
        Vec2::new(1.0, 1.0),
        Vec2::new(1.0, 0.0),


        Vec2::new(0.0, 0.0),
        Vec2::new(0.0, 1.0),
        Vec2::new(1.0, 1.0),
    ];

    (vertices, tex_coords)
}