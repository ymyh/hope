use glam::{Vec4, IVec2, Vec2};
use hope::gl::enums::glBufferBit::GLBufferBit;
use hope::gl::glColor::GLColor;
use hope::gl::{glContext::GLContext, shader::program::Program};
use hope::gl::shader::shader::Shader;
use hope::make_color;
use image::ColorType;
use shader::Shader;
use varying::Varying;
use hope::gl::shader::varying::Varying;

#[derive(Default, Shader)]
struct TriangleShader
{
    #[varying(ColorVarying)]
    varyings: Vec<ColorVarying>,

    //下面这两个是attribute
    position: Vec<Vec2>,
    colors: Vec<GLColor>,
}

impl Program<ColorVarying> for TriangleShader
{
    fn vertex(&mut self, index: usize) -> Vec4
    {
        //向varying写入值
        self.varyings.push(ColorVarying { color: self.colors[index] });
        Vec4::from((self.position[index], 0.0, 1.0))
    }

    //第一个参数是已经插值好的数据，第二个是像素的位置
    fn fragment(&mut self, varying: &ColorVarying, _: IVec2) -> GLColor
    {
        varying.color
    }

    //暂时用不到
    fn sample(&mut self, _: &ColorVarying)
    {

    }
}

#[derive(Clone, Copy, Default, Varying)]
struct ColorVarying
{
    color: GLColor,
}

fn main()
{
    let mut gl = GLContext::new(1280, 720);
    let mut fb = gl.create_buffer();
    let mut shader = TriangleShader::default();

    //添加颜色附着
    fb.attach_color();

    shader.position = vec![
        Vec2::new(0.0, 0.5),
        Vec2::new(-0.5, -0.5),
        Vec2::new(0.5, -0.5),
    ];

    shader.colors = vec![make_color!(255, 0, 0), make_color!(0, 255, 0), make_color!(0, 0, 255)];

    gl.clear_color(make_color!(255));
    gl.clear(GLBufferBit::Color, &mut fb);

    gl.draw_arrays(&mut shader, 3, 0, &mut fb);

    image::save_buffer("./triangle.png", fb.get_color_buffer(), 1280, 720, ColorType::Rgba8).unwrap()
}