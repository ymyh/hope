use std::sync::Arc;

use glam::{Vec2, Vec4, IVec2, Mat4, Vec3};
use hope::gl::enums::glBlendFunc::GLBlendFunc;
use hope::gl::enums::glBufferBit::GLBufferBit;
use hope::gl::enums::glCompareFunc::GLCompareFunc;
use hope::gl::enums::glFunction::GLFunction;
use hope::gl::enums::glStencilOp::GLStencilOp;
use hope::gl::glColor::GLColor;
use hope::gl::glContext::GLContext;
use hope::gl::glTexture::GLTexture;
use hope::gl::sampler::sampler2d::Sampler2D;
use hope::gl::shader::program::Program;
use hope::gl::util::read_image;
use image::ColorType;
use shader::Shader;
use varying::Varying;
use hope::gl::shader::shader::Shader;
use hope::gl::shader::varying::Varying;

//画圆形着色器
#[derive(Default, Shader)]
struct CircleShader
{
    #[varying(CircleVarying)]
    varyings: Vec<CircleVarying>,

    positions: Vec<Vec4>,
    tex_coords: Vec<Vec2>,

    #[uniform]
    mvp: Mat4,

    #[sampler]
    circle: Sampler2D,
}

impl Program<CircleVarying> for CircleShader
{
    fn vertex(&mut self, index: usize) -> Vec4
    {
        self.varyings.push(CircleVarying { tex_coords: self.tex_coords[index] });
        self.mvp * self.positions[index]
    }

    fn fragment(&mut self, _: &CircleVarying, _: IVec2) -> hope::gl::glColor::GLColor
    {
        self.circle.get_color()
    }

    fn sample(&mut self, varying: &CircleVarying)
    {
        self.circle.sample(varying.tex_coords);
    }
}

#[derive(Default, Clone, Copy, Varying)]
struct CircleVarying
{
    tex_coords: Vec2,
}

//纯色着色器
#[derive(Default, Shader)]
struct PureColorShader
{
    #[varying(PureColorVarying)]
    varying: Vec<PureColorVarying>,

    positions: Vec<Vec4>,

    #[uniform]
    mvp: Mat4,

    #[uniform]
    color: Vec4,
}

impl Program<PureColorVarying> for PureColorShader
{
    fn vertex(&mut self, index: usize) -> Vec4
    {
        self.varying.push(PureColorVarying { });
        self.mvp * self.positions[index]
    }

    fn fragment(&mut self, _: &PureColorVarying, _: IVec2) -> GLColor
    {
        self.color.into()
    }

    fn sample(&mut self, _: &PureColorVarying)
    {
        
    }
}

#[derive(Default, Clone, Copy, Varying)]
struct PureColorVarying
{
    
}

fn main()
{
    let mut gl = GLContext::new(1920, 1080);
    let mut fb = gl.create_buffer();
    let mut circle_shader = CircleShader::default();
    let mut pure_color_shader = PureColorShader::default();

    //添加附着
    fb.attach_color();
    fb.attach_stencil();

    gl.enable(GLFunction::AlphaTest);
    gl.alpha_func(GLCompareFunc::Greater, 0);

    gl.enable(GLFunction::StencilTest);
    gl.stencil_op(GLStencilOp::Keep, GLStencilOp::Keep, GLStencilOp::Replace);
    gl.blend_func(GLBlendFunc::SrcAlpha, GLBlendFunc::OneMinusSrcAlpha);

    let view = Mat4::orthographic_rh(-440.0, 1280.0 - 440.0, 720.0, 0.0, 400.0, -400.0);

    let tr1 = Mat4::from_translation(Vec3::new(0.0, 100.0, 0.0));
    let tr2 = Mat4::from_translation(Vec3::new(150.0, 300.0, 0.0));
    let tr3 = Mat4::from_translation(Vec3::new(-150.0, 300.0, 0.0));

    (circle_shader.positions, circle_shader.tex_coords) = make_quad(400.0, 400.0);
    pure_color_shader.positions = circle_shader.positions.clone();

    let circle = GLTexture::from_bytes(&read_image("./img/circle.png"), 400, 400).unwrap();
    circle_shader.circle = Sampler2D::new(Arc::new(circle)); 

    //第一个圆
    gl.color_mask(false, false, false, false);
    gl.clear(GLBufferBit::Color | GLBufferBit::Stencil, &mut fb);
    gl.disable(GLFunction::Blend);
    gl.stencil_func(GLCompareFunc::Always, 0x01, 0xFF);

    circle_shader.mvp = view * tr1;

    gl.draw_arrays(&mut circle_shader, 6, 0, &mut fb);

    gl.color_mask(true, true, true, true);
    gl.stencil_func(GLCompareFunc::Equal, 0x01, 0xFF);
    gl.enable(GLFunction::Blend);

    pure_color_shader.mvp = view * tr1;
    pure_color_shader.color = Vec4::new(1.0, 0.0, 0.0, 0.9);

    gl.draw_arrays(&mut pure_color_shader, 6, 0, &mut fb);

    //第二个圆
    gl.color_mask(false, false, false, false);
    gl.clear(GLBufferBit::Stencil, &mut fb);
    gl.disable(GLFunction::Blend);
    gl.stencil_func(GLCompareFunc::Always, 0x01, 0xFF);

    circle_shader.mvp = view * tr2;

    gl.draw_arrays(&mut circle_shader, 6, 0, &mut fb);

    gl.color_mask(true, true, true, true);
    gl.stencil_func(GLCompareFunc::Equal, 0x01, 0xFF);
    gl.enable(GLFunction::Blend);

    pure_color_shader.mvp = view * tr2;
    pure_color_shader.color = Vec4::new(0.0, 1.0, 0.0, 0.7);

    gl.draw_arrays(&mut pure_color_shader, 6, 0, &mut fb);

    //第三个圆
    gl.color_mask(false, false, false, false);
    gl.clear(GLBufferBit::Stencil, &mut fb);
    gl.disable(GLFunction::Blend);
    gl.stencil_func(GLCompareFunc::Always, 0x01, 0xFF);

    circle_shader.mvp = view * tr3;

    gl.draw_arrays(&mut circle_shader, 6, 0, &mut fb);

    gl.color_mask(true, true, true, true);
    gl.stencil_func(GLCompareFunc::Equal, 0x01, 0xFF);
    gl.enable(GLFunction::Blend);

    pure_color_shader.mvp = view * tr3;
    pure_color_shader.color = Vec4::new(0.0, 0.0, 1.0, 0.7);
    gl.draw_arrays(&mut pure_color_shader, 6, 0, &mut fb);

    image::save_buffer("./circles.png", fb.get_color_buffer(), 1920, 1080, ColorType::Rgba8).unwrap()
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