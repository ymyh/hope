use glam::{Vec4, IVec2, Mat4, EulerRot, Vec3};
use hope::gl::enums::glBufferBit::GLBufferBit;
use hope::gl::enums::glCompareFunc::GLCompareFunc;
use hope::gl::enums::glFunction::GLFunction;
use hope::gl::glColor::GLColor;
use hope::gl::shader::attribute::Attribute;
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
    #[varying(ColorVarying)]
    varyings: Vec<ColorVarying>,

    position: Vec<Vec4>,

    #[attribute]
    colors: Attribute<Vec4>,

    #[uniform]
    mvp: Mat4,
}

impl Program<ColorVarying> for TestShader
{
    fn vertex(&mut self, index: usize) -> Vec4
    {
        self.varyings.push(ColorVarying { color: self.colors.get() });
        self.mvp * self.position[index]
    }

    fn fragment(&mut self, varying: &ColorVarying, _: IVec2) -> GLColor
    {
        varying.color.into()
    }

    fn sample(&mut self, _: &ColorVarying)
    {

    }
}

#[derive(Clone, Copy, Default, Varying)]
struct ColorVarying
{
    color: Vec4,
}

fn main()
{
    let mut gl = GLContext::new(1280, 720);
    let mut fb = gl.create_buffer();
    let mut shader = TestShader::default();

    gl.enable(GLFunction::CullFace);
    gl.enable(GLFunction::DepthTest);

    //添加颜色和深度附着
    fb.attach_color();
    fb.attach_depth();

    let (vertices, colors) = make_f();
    shader.position = vertices;
    shader.colors = Attribute::new(colors, 6);
    
    let proj = Mat4::perspective_rh(60f32.to_radians(), 1280. / 720., 1., 2000.);
    let view = Mat4::look_at_rh(Vec3::new(0.0, 0.0, -1.0), Vec3::new(0.0, 0.0, 0.0), Vec3::new(0.0, -1.0, 0.0)).inverse();
    let model = Mat4::from_translation(Vec3::new(-100.0, -50.0, 250.0)) *
    Mat4::from_euler(EulerRot::ZXY, 0f32.to_radians(), 15f32.to_radians(), 45f32.to_radians());
    shader.mvp = proj * view * model;

    gl.clear_color(make_color!(255));
    gl.clear_depth(-1.);
    gl.depth_value(GLFunction::Reciprocal_W);   //使用w的倒数作为深度，默认是使用z
    gl.depth_func(GLCompareFunc::Greater);
    gl.clear(GLBufferBit::Color | GLBufferBit::Depth, &mut fb);
    gl.enable(GLFunction::AlphaTest);

    gl.draw_arrays(&mut shader, 6 * 16, 0, &mut fb);

    image::save_buffer("./f.png", fb.get_color_buffer(), 1280, 720, ColorType::Rgba8).unwrap()
}

fn make_f() -> (Vec<Vec4>, Vec<Vec4>)
{
    let raw_vertices = vec![
    // left column front
    0,   0,  0,
    0, 150,  0,
    30,   0,  0,
    0, 150,  0,
    30, 150,  0,
    30,   0,  0,

    // top rung front
    30,   0,  0,
    30,  30,  0,
    100,   0,  0,
    30,  30,  0,
    100,  30,  0,
    100,   0,  0,

    // middle rung front
    30,  60,  0,
    30,  90,  0,
    67,  60,  0,
    30,  90,  0,
    67,  90,  0,
    67,  60,  0,

    // left column back
      0,   0,  30,
     30,   0,  30,
      0, 150,  30,
      0, 150,  30,
     30,   0,  30,
     30, 150,  30,

    // top rung back
     30,   0,  30,
    100,   0,  30,
     30,  30,  30,
     30,  30,  30,
    100,   0,  30,
    100,  30,  30,

    // middle rung back
     30,  60,  30,
     67,  60,  30,
     30,  90,  30,
     30,  90,  30,
     67,  60,  30,
     67,  90,  30,

    // top
      0,   0,   0,
    100,   0,   0,
    100,   0,  30,
      0,   0,   0,
    100,   0,  30,
      0,   0,  30,

    // top rung right
    100,   0,   0,
    100,  30,   0,
    100,  30,  30,
    100,   0,   0,
    100,  30,  30,
    100,   0,  30,

    // under top rung
    30,   30,   0,
    30,   30,  30,
    100,  30,  30,
    30,   30,   0,
    100,  30,  30,
    100,  30,   0,

    // between top rung and middle
    30,   30,   0,
    30,   60,  30,
    30,   30,  30,
    30,   30,   0,
    30,   60,   0,
    30,   60,  30,

    // top of middle rung
    30,   60,   0,
    67,   60,  30,
    30,   60,  30,
    30,   60,   0,
    67,   60,   0,
    67,   60,  30,

    // right of middle rung
    67,   60,   0,
    67,   90,  30,
    67,   60,  30,
    67,   60,   0,
    67,   90,   0,
    67,   90,  30,

    // bottom of middle rung.
    30,   90,   0,
    30,   90,  30,
    67,   90,  30,
    30,   90,   0,
    67,   90,  30,
    67,   90,   0,

    // right of bottom
    30,   90,   0,
    30,  150,  30,
    30,   90,  30,
    30,   90,   0,
    30,  150,   0,
    30,  150,  30,

    // bottom
    0,   150,   0,
    0,   150,  30,
    30,  150,  30,
    0,   150,   0,
    30,  150,  30,
    30,  150,   0,

    // left side
    0,   0,   0,
    0,   0,  30,
    0, 150,  30,
    0,   0,   0,
    0, 150,  30,
    0, 150,   0];

    let raw_colors = [
        make_color!(200, 70, 120),
        make_color!(200, 70, 120),
        make_color!(200, 70, 120),

        make_color!(80, 70, 200),
        make_color!(80, 70, 200),
        make_color!(80, 70, 200),

        make_color!(70, 200, 210),
        make_color!(200, 200, 70),
        make_color!(210, 100, 70),
        make_color!(210, 160, 70),
        make_color!(70, 180, 210),
        make_color!(100, 70, 210),
        make_color!(76, 210, 100),
        make_color!(140, 210, 80),
        make_color!(90, 130, 110),
        make_color!(160, 160, 220),
    ];

    let mut i = 0;
    let mut vertices = Vec::with_capacity(raw_vertices.len() / 3);

    while i < raw_vertices.len()
    {
        vertices.push(Vec4::new(raw_vertices[i] as f32, raw_vertices[i + 1] as f32, raw_vertices[i + 2] as f32, 1.));
        i += 3;
    }

    let mut colors = Vec::with_capacity(raw_colors.len());

    i = 0;
    while i < raw_colors.len()
    {
        colors.push(raw_colors[i].into());
        i += 1;
    }

    (vertices, colors)
}