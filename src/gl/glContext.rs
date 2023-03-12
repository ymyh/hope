use std::{ptr::addr_of_mut};

use glam::{Vec4, IVec2, Vec3, Vec3A, Vec2, UVec2};
use threadPool::ThreadPool;

use super::{shader::{program::Program, varying::Varying, shader::Shader}, enums::{glFunction::GLFunction, glCompareFunc::GLCompareFunc, glSamplePoint::GLSamplePoint, glBufferBit::GLBufferBit, glStencilOp::GLStencilOp, glBlendFunc::GLBlendFunc, glBlendEquation::GLBlendEquation}, glFrameBuffer::GLFrameBuffer, glColor::GLColor, util::{is_between, div_255}};

pub struct GLContext
{
    width: u32,
    height: u32,

    viewport_min: UVec2,
    viewport_max: UVec2,

    vertices_pool: Vec<Vec<Vec4>>,

    anisotropic_filter: GLSamplePoint,

    pool: ThreadPool,

    color_mask: u32,

    alpha_test: bool,
    alpha_func: GLCompareFunc,
    alpha_ref: u8,

    depth_test: bool,
    depth_mask: u32,
    depth_func: GLCompareFunc,
    depth_value: GLFunction,

    stencil_test: bool,
    stencil_write_mask: u8,
    stencil_func: GLCompareFunc,
    stencil_ref: u8,
    stencil_test_mask: u8,

    stencil_fail_op: GLStencilOp,
    depth_fail_op: GLStencilOp,
    all_pass_op: GLStencilOp,

    blend: bool,
    blend_src_func: GLBlendFunc,
    blend_dst_func: GLBlendFunc,
    blend_equation: GLBlendEquation,
    blend_color: GLColor,

    async_draw: bool,
    cull_face: bool,
    front_face_is_ccw: bool,

    color: GLColor,
    depth: f32,
    stencil: u8,
}

impl GLContext
{
    pub fn new(width: u32, height: u32) -> Self
    {
        return Self {
            width,
            height,

            viewport_min: UVec2::default(),
            viewport_max: UVec2::new(width, height) - 1,

            vertices_pool: Vec::new(),

            anisotropic_filter: GLSamplePoint::X1,

            pool: ThreadPool::new(0),

            color_mask: 0xFFFFFFFF,

            alpha_test: false,
            alpha_func: GLCompareFunc::Greater,
            alpha_ref: 0x00,

            depth_test: false,
            depth_mask: 0xFFFFFFFF,
            depth_func: GLCompareFunc::Less,
            depth_value: GLFunction::Z,

            stencil_test: false,
            stencil_write_mask: 0xFF,
            stencil_ref: 0x00,
            stencil_test_mask: 0xFF,
            stencil_func: GLCompareFunc::Always,

            stencil_fail_op: GLStencilOp::Keep,
            depth_fail_op: GLStencilOp::Keep,
            all_pass_op: GLStencilOp::Keep,

            blend: false,
            blend_src_func: GLBlendFunc::One,
            blend_dst_func: GLBlendFunc::Zero,
            blend_equation: GLBlendEquation::Add,
            blend_color: GLColor::ONE,

            async_draw: false,
            cull_face: false,
            front_face_is_ccw: true,

            color: GLColor::ONE,
            depth: 1.0,
            stencil: 0x00,
        }
    }

    pub fn viewport(&mut self, x: u32, y: u32, width: u32, height: u32)
    {
        self.width = width;
        self.height = height;

        self.viewport_min = UVec2::new(x, y);
        self.viewport_max = UVec2::new(x + width, y + height) - 1;
    }

    pub fn clear_color(&mut self, color: GLColor)
    {
        self.color = color;
    }

    pub fn clear_depth(&mut self, depth: f32)
    {
        self.depth = depth;
    }

    pub fn clear_stencil(&mut self, stencil: u8)
    {
        self.stencil = stencil;
    }

    pub fn clear(&self, bits: GLBufferBit, fb: &mut GLFrameBuffer)
    {
        if bits & GLBufferBit::Color == GLBufferBit::Color
        {
            fb.do_clear_color(self.color);
        }

        if bits & GLBufferBit::Depth == GLBufferBit::Depth
        {
            fb.do_clear_depth(self.depth);
        }

        if bits & GLBufferBit::Stencil == GLBufferBit::Stencil
        {
            fb.do_clear_stencil(self.stencil);
        }
    }

    pub fn enable(&mut self, func: GLFunction) -> bool
    {
        self.switch_function(func, true)
    }

    pub fn disable(&mut self, func: GLFunction) -> bool
    {
        self.switch_function(func, false)
    }

    fn switch_function(&mut self, func: GLFunction, status: bool) -> bool
    {
        match func
        {
            GLFunction::AlphaTest =>
            {
                self.alpha_test = status;
            }

            GLFunction::Blend =>
            {
                self.blend = status;
            }

            GLFunction::CullFace =>
            {
                self.cull_face = status;
            }

            GLFunction::DepthTest =>
            {
                self.depth_test = status;
            }

            GLFunction::StencilTest =>
            {
                self.stencil_test = status;
            }

            _ => 
            {
                return false;
            }
        }

        true
    }

    /// 设置绘制线程数量
    pub fn thread_count(&mut self, thread: u32)
    {
        if thread == 1
        {
            self.async_draw = false;
        }
        else
        {
            self.async_draw = true;
            self.pool = ThreadPool::new(thread);
        }
    }

    pub fn alpha_func(&mut self, func: GLCompareFunc, alpha: u8)
    {
        self.alpha_func = func;
        self.alpha_ref = alpha;
    }

    pub fn color_mask(&mut self, r: bool, g: bool, b: bool, a: bool)
    {
        self.color_mask = 0xFFFFFFFF;

        self.color_mask &= if r { 0xFFFFFFFF } else { 0xFFFFFF00 };
        self.color_mask &= if g { 0xFFFFFFFF } else { 0xFFFF00FF };
        self.color_mask &= if b { 0xFFFFFFFF } else { 0xFF00FFFF };
        self.color_mask &= if a { 0xFFFFFFFF } else { 0x00FFFFFF };
    }

    pub fn depth_func(&mut self, func: GLCompareFunc)
    {
        self.depth_func = func;
    }

    pub fn depth_mask(&mut self, mask: bool)
    {
        self.depth_mask = if mask { 0xFFFFFFFF } else { 0x00000000 };
    }

    /// 设置深度值组件
    pub fn depth_value(&mut self, value: GLFunction)
    {
        if value == GLFunction::Z || value == GLFunction::Reciprocal_W
        {
            self.depth_value = value;
        }
        else
        {
            eprintln!("无效的深度值枚举");
        }
    }

    pub fn stencil_mask(&mut self, mask: u8)
    {
        self.stencil_write_mask = mask;
    }

    pub fn stencil_func(&mut self, func: GLCompareFunc, ref_value: u8, mask: u8)
    {
        self.stencil_func = func;
        self.stencil_ref = ref_value;
        self.stencil_test_mask = mask;
    }

    pub fn stencil_op(&mut self, stencil_fail: GLStencilOp, depth_fail: GLStencilOp, all_pass: GLStencilOp)
    {
        self.stencil_fail_op = stencil_fail;
        self.depth_fail_op = depth_fail;
        self.all_pass_op = all_pass;
    }

    /// 设置前面是不是逆时针三角形
    pub fn front_face(&mut self, ccw: bool)
    {
        self.front_face_is_ccw = ccw;
    }

    /// 设置各向异性过滤采样点
    pub fn anisotropic_filter(&mut self, sample_point: GLSamplePoint)
    {
        self.anisotropic_filter = sample_point;
    }

    pub fn blend_func(&mut self, src_factor: GLBlendFunc, dst_factor: GLBlendFunc)
    {
        self.blend_src_func = src_factor;
        self.blend_dst_func = dst_factor;
    }

    pub fn blend_equation(&mut self, equation: GLBlendEquation)
    {
        self.blend_equation = equation;
    }

    pub fn blend_color(&mut self, color: GLColor)
    {
        self.blend_color = color;
    }

    ///根据自身大小创建同等大小的帧缓冲
    pub fn create_buffer(&self) -> GLFrameBuffer
    {
        GLFrameBuffer::new(self.width as usize, self.height as usize)
    }

    fn cull_face(&self, vertices: &[Vec4]) -> bool
    {
        let edge1 = Vec3::new(vertices[1].x - vertices[0].x, vertices[1].y - vertices[0].y, vertices[1].z - vertices[0].z);
        let edge2 = Vec3::new(vertices[2].x - vertices[0].x, vertices[2].y - vertices[0].y, vertices[2].z - vertices[0].z);

        let view = edge1.cross(edge2);

        // 视线方向是 (0, 0, 1)
        if self.front_face_is_ccw { view.z < 0. } else { view.z > 0. }
    }

    fn do_blend_color(&self, mut src_color: GLColor, mut dst_color: GLColor) -> GLColor
    {
        // println!("{:?}, {:?}", src_color, dst_color);
        src_color = match self.blend_src_func
        {
            GLBlendFunc::Zero => GLColor::ZERO,
            GLBlendFunc::One => src_color,

            GLBlendFunc::SrcAlpha => src_color * src_color.a,
            GLBlendFunc::DstAlpha => src_color * dst_color.a,
            GLBlendFunc::OneMinusSrcAlpha => src_color * (255 - src_color.a),
            GLBlendFunc::OneMinusDstAlpha => src_color * (255 - dst_color.a),

            GLBlendFunc::SrcColor => src_color * src_color,
            GLBlendFunc::DstColor => src_color * dst_color,

            GLBlendFunc::OneMinusSrcColor => src_color * (GLColor::ONE - src_color),
            GLBlendFunc::OneMinusDstColor => src_color * (GLColor::ONE - dst_color),

            GLBlendFunc::ConstColor => src_color * self.blend_color,
            GLBlendFunc::OneMinusConstColor => src_color * (GLColor::ONE - self.blend_color),
            GLBlendFunc::ConstAlpha => src_color * self.blend_color.a,
            GLBlendFunc::OneMinusConstAlpha => src_color * (255 - self.blend_color.a),
        };

        dst_color = match self.blend_dst_func
        {
            GLBlendFunc::Zero => GLColor::ZERO,
            GLBlendFunc::One => dst_color,

            GLBlendFunc::SrcAlpha => dst_color * src_color.a,
            GLBlendFunc::DstAlpha => dst_color * dst_color.a,
            GLBlendFunc::OneMinusSrcAlpha => dst_color * (255 - src_color.a),
            GLBlendFunc::OneMinusDstAlpha => dst_color * (255 - dst_color.a),

            GLBlendFunc::SrcColor => dst_color * src_color,
            GLBlendFunc::DstColor => dst_color * dst_color,

            GLBlendFunc::OneMinusSrcColor => dst_color * (GLColor::ONE - src_color),
            GLBlendFunc::OneMinusDstColor => dst_color * (GLColor::ONE - dst_color),

            GLBlendFunc::ConstColor => dst_color * self.blend_color,
            GLBlendFunc::OneMinusConstColor => dst_color * (GLColor::ONE - self.blend_color),
            GLBlendFunc::ConstAlpha => dst_color * self.blend_color.a,
            GLBlendFunc::OneMinusConstAlpha => dst_color * (255 - self.blend_color.a),
        };

        let r = match self.blend_equation
        {
            GLBlendEquation::Add => src_color + dst_color,
            GLBlendEquation::Subtract => src_color - dst_color,
            GLBlendEquation::ReverseSubtract => dst_color - src_color,
            GLBlendEquation::Min => src_color.min(dst_color),
            GLBlendEquation::Max => src_color.max(dst_color),
        };

        
        r
    }

    pub fn draw_arrays<S: Program<T> + Shader<T> + Clone + Send, T: Varying>
        (&mut self, shader: &mut S, count: usize, offset: usize, fb: &mut GLFrameBuffer)
    {
        shader.reset();

        let mut vertices = self.vertex_phase(shader, count, offset);

        self.pixel_phase(shader, &vertices, fb);

        vertices.clear();
        self.vertices_pool.push(vertices);
    }

    /// 顶点着色器阶段
    fn vertex_phase<S: Program<T> + Shader<T>, T: Varying>(&mut self, shader: &mut S, count: usize, offset: usize) -> Vec<Vec4>
    {
        for _ in 0..offset
        {
            shader.next();
        }

        let count = count - count % 3;
        let mut vertices;

        vertices = self.vertices_pool.pop().unwrap_or(Vec::with_capacity(count));
        vertices.clear();

        for i in 0..count
        {
            vertices.push(shader.vertex(i + offset));

            if (i + 1) % 3 == 0
            {
                let len = vertices.len();
                let vert = &mut vertices[len - 3..len];

                //剔除w <= 0
                if vert[0].w <= 0. || vert[1].w <= 0. || vert[2].w <= 0.
                {
                    vertices.truncate(len - 3);
                }
                else
                {
                    let rhw = 1. / vert[0].w;
                    vert[0] *= rhw;
                    vert[0].w = rhw;

                    let rhw = 1. / vert[1].w;
                    vert[1] *= rhw;
                    vert[1].w = rhw;

                    let rhw = 1. / vert[2].w;
                    vert[2] *= rhw;
                    vert[2].w = rhw;
                }
            }

            shader.next();
        }

        vertices
    }

    /// 片段着色器阶段
    fn pixel_phase<S: Program<T> + Shader<T> + Clone + Send, T: Varying>(&mut self, shader: &mut S, vertices: &[Vec4], fb: &mut GLFrameBuffer)
    {
        let mut i = 0;

        let count = self.pool.thread_count() as usize;
        let mut shaders = Vec::with_capacity(count);

        if self.async_draw
        {
            for _ in 0..count
            {
                shaders.push(shader.clone());
            }
        }

        while i != vertices.len()
        {
            let vert = &vertices[i..i+3];
            let varying = unsafe { &(*(shader as *const S)).get_varying()[i..i+3] };

            if self.cull_face
            {
                if self.cull_face(vert)
                {
                    i += 3;
                    continue;
                }
            }

            if self.async_draw
            {
                self.triangle_multi_thread(&mut shaders, varying, vert, fb);
            }
            else
            {
                self.triangle(shader, varying, vert, fb);
            }
            
            i += 3;
        }
    }

    #[unchecked::unchecked]
    fn triangle<S: Program<T> + Shader<T>, T: Varying>(&self, shader: &mut S, varying: &[T], vertices: &[Vec4], fb: &mut GLFrameBuffer)
    {
        let mut min = self.viewport_max.as_ivec2();
        let mut max = self.viewport_min.as_ivec2();

        let vert = [
            Vec2::new((1. + vertices[0].x) * self.width as f32 * 0.5 + max.x as f32,
            (1. - vertices[0].y) * self.height as f32 * 0.5 + max.y as f32),

            Vec2::new((1. + vertices[1].x) * self.width as f32 * 0.5 + max.x as f32,
            (1. - vertices[1].y) * self.height as f32 * 0.5 + max.y as f32),

            Vec2::new((1. + vertices[2].x) * self.width as f32 * 0.5 + max.x as f32,
            (1. - vertices[2].y) * self.height as f32 * 0.5 + max.y as f32)
        ];

        for v in vert
        {
            min.x = i32::min(min.x, v.x as i32);
            min.y = i32::min(min.y, v.y as i32);

            max.x = i32::max(max.x, v.x as i32);
            max.y = i32::max(max.y, v.y as i32);
        }

        min = IVec2::max(min, self.viewport_min.as_ivec2());
        max = IVec2::min(max, self.viewport_max.as_ivec2());

        let rhw0 = vertices[0].w;
        let rhw1 = vertices[1].w;
        let rhw2 = vertices[2].w;

        let v0 = varying[0] * rhw0;
        let v1 = varying[1] * rhw1;
        let v2 = varying[2] * rhw2;

        let z0;
        let z1;
        let z2;

        if self.depth_value == GLFunction::Reciprocal_W
        {
            z0 = 0.;
            z1 = 0.;
            z2 = 0.;
        }
        else
        {
            z0 = vertices[0].z * rhw0;
            z1 = vertices[1].z * rhw1;
            z2 = vertices[2].z * rhw2;
        }

        let a = Vec3A::new(vert[2].x - vert[0].x,
            vert[1].x - vert[0].x,
            vert[0].x);

        let b = Vec3A::new(vert[2].y - vert[0].y,
            vert[1].y - vert[0].y,
            vert[0].y);

        let mut valid;
        let mut some_test_failed;

        let mut varyings = [T::default(); 4];
        let mut screens = [Vec3A::default(); 4];
        let mut zs = [0.; 4];
        let mut ws = [0.; 4];

        let mut y = min.y;

        // let do_stencil_op_ptr = if self.stencil_test { do_stencil_op } else { _do_stencil_op_empty };
        
        while y <= max.y
        {
            let mut x = min.x;
            let mut inside = false;

            while x <= max.x
            {
                let mut quad_x = x as f32 + 0.5;
                let mut quad_y = y as f32 + 0.5;

                valid = 0;
                some_test_failed = false;

                //一次性处理四个像素
                for i in 0..4
                {
                    let screen = barycentric(a, b, Vec2 { x: quad_x, y: quad_y });
                    let rhw = rhw0 * screen.x + rhw1 * screen.y + rhw2 * screen.z;
                    let w = 1. / rhw;
                    screens[i] = screen;
                    ws[i] = w;

                    let xx = quad_x as i32;
                    let yy = quad_y as i32;

                    if is_between(xx, min.x, max.x) && is_between(yy, min.y, max.y)
                    {
                        //三角形重心坐标出现负值，证明在三角形外
                        if screen.x < 0. || screen.y < 0. || screen.z < 0.
                        {

                        }
                        else
                        {
                            let depth = if self.depth_value == GLFunction::Reciprocal_W
                            { rhw }
                            else
                            { (z0 * screen.x + z1 * screen.y + z2 * screen.z) * w };
                            zs[i] = depth;

                            if !self.alpha_test
                            {
                                if let Some(depth_test_failed) = self.do_stencil_depth_test(xx, yy, i as i32, &mut valid, depth, fb)
                                {
                                    some_test_failed = depth_test_failed;
                                }
                                else
                                {
                                    some_test_failed = true;
                                }
                            }
                            else
                            {
                                valid |= 1 << i;
                            }

                            inside = true;
                        }
                    }

                    //0-0 1-1 2-0 3-1
                    quad_x += if i % 2 == 0 { 1. } else { -1. };
                    //0-0 1-0 2-1 3-1
                    quad_y += if i % 2 == 0 { 0. } else {  1. };
                }

                if valid != 0
                {
                    //片段着色器分为两部分，sample部分是给纹理采样用的，在这里采样器提前算好mipmap等级
                    for i in 0..4
                    {
                        varyings[i] = (v0 * screens[i].x + v1 * screens[i].y + v2 * screens[i].z) * ws[i];
                        shader.sample(&varyings[i]);
                    }

                    shader.compute_level(self.anisotropic_filter as i32);

                    let mut xx = x;
                    let mut yy = y;

                    for i in 0..4
                    {
                        let color = shader.fragment(&varyings[i], IVec2::new(xx, yy));

                        if valid & (1 << i) != 0
                        {
                            if self.alpha_test
                            {
                                if compare_value(self.alpha_func, self.alpha_ref, color.a)
                                {
                                    match self.do_stencil_depth_test(xx, yy, i as i32, &mut valid, zs[i], fb)
                                    {
                                        //模板或者深度测试不通过
                                        None |
                                        Some(true) =>
                                        {
                                            //0-0 1-1 2-0 3-1
                                            xx += if i % 2 == 0 { 1 } else { -1 };
                                            //0-0 1-0 2-1 3-1
                                            yy += if i % 2 == 0 { 0 } else {  1 };
                                            continue;
                                        }

                                        _ => {}
                                    }
                                }
                                else
                                {
                                    //0-0 1-1 2-0 3-1
                                    xx += if i % 2 == 0 { 1 } else { -1 };
                                    //0-0 1-0 2-1 3-1
                                    yy += if i % 2 == 0 { 0 } else {  1 };
                                    continue;
                                }
                            }

                            let keep_one: u32 = fb.get_color(xx, yy).into();

                            //混合
                            if self.blend
                            {
                                let color: u32 = self.do_blend_color(color, fb.get_color(xx, yy)).into();
                                fb.set_color(xx, yy, GLColor::from((color & self.color_mask) | (keep_one & !self.color_mask)));
                            }
                            else
                            {
                                let color: u32 = color.into();
                                fb.set_color(xx, yy, GLColor::from((color & self.color_mask) | (keep_one & !self.color_mask)));
                            }
                        }

                        //0-0 1-1 2-0 3-1
                        xx += if i % 2 == 0 { 1 } else { -1 };
                        //0-0 1-0 2-1 3-1
                        yy += if i % 2 == 0 { 0 } else {  1 };
                    }
                }
                else if inside && !some_test_failed
                {
                    //从三角形出来后后续的像素肯定都不属于这个三角形
                    break;
                }

                x += 2;
            }

            y += 2;
        }
    }

    //基本同上
    #[unchecked::unchecked]
    fn triangle_multi_thread<S: Program<T> + Shader<T> + Clone + Send, T: Varying + Send + Sync>(&mut self, shaders: &mut Vec<S>, varying: &[T], vertices: &[Vec4], fb: &mut GLFrameBuffer)
    {
        let mut min = self.viewport_max.as_ivec2();
        let mut max = self.viewport_min.as_ivec2();

        let vert = [
            Vec2::new((1. + vertices[0].x) * self.width as f32 * 0.5 + max.x as f32,
            (1. - vertices[0].y) * self.height as f32 * 0.5 + max.y as f32),

            Vec2::new((1. + vertices[1].x) * self.width as f32 * 0.5 + max.x as f32,
            (1. - vertices[1].y) * self.height as f32 * 0.5 + max.y as f32),

            Vec2::new((1. + vertices[2].x) * self.width as f32 * 0.5 + max.x as f32,
            (1. - vertices[2].y) * self.height as f32 * 0.5 + max.y as f32)
        ];

        for v in vert
        {
            min.x = i32::min(min.x, v.x as i32);
            min.y = i32::min(min.y, v.y as i32);

            max.x = i32::max(max.x, v.x as i32);
            max.y = i32::max(max.y, v.y as i32);
        }

        min = IVec2::max(min, self.viewport_min.as_ivec2());
        max = IVec2::min(max, self.viewport_max.as_ivec2());

        let rhw0 = vertices[0].w;
        let rhw1 = vertices[1].w;
        let rhw2 = vertices[2].w;

        let v0 = varying[0] * rhw0;
        let v1 = varying[1] * rhw1;
        let v2 = varying[2] * rhw2;

        let z0;
        let z1;
        let z2;

        if self.depth_value == GLFunction::Reciprocal_W
        {
            z0 = 0.;
            z1 = 0.;
            z2 = 0.;
        }
        else
        {
            z0 = vertices[0].z * rhw0;
            z1 = vertices[1].z * rhw1;
            z2 = vertices[2].z * rhw2;
        }

        let a = Vec3A::new(vert[2].x - vert[0].x,
            vert[1].x - vert[0].x,
            vert[0].x);

        let b = Vec3A::new(vert[2].y - vert[0].y,
            vert[1].y - vert[0].y,
            vert[0].y);

        let pool = unsafe { &mut *addr_of_mut!(self.pool) };
        let threads = pool.thread_count() as i32;
        let this = self as *const _ as usize;

        // let do_stencil_op_ptr = if self.stencil_test { do_stencil_op } else { _do_stencil_op_empty };
        
        pool.scope(|s|
        {
            for i in 0..threads
            {
                let shader = &mut shaders[i as usize];
                let fb = unsafe { &mut *(fb as *mut GLFrameBuffer) };

                let mut y = min.y + i * 2;

                s.spawn(move ||
                {
                    let this = unsafe { std::mem::transmute::<usize, &GLContext>(this) };

                    let mut varyings = [T::default(); 4];
                    let mut screens = [Vec3A::default(); 4];
                    let mut zs = [0.; 4];
                    let mut ws = [0.; 4];

                    let mut valid;
                    let mut some_test_failed;

                    while y <= max.y
                    {
                        let mut x = min.x;
                        let mut inside = false;

                        while x <= max.x
                        {
                            let mut quad_x = x as f32 + 0.5;
                            let mut quad_y = y as f32 + 0.5;

                            valid = 0;
                            some_test_failed = false;

                            for i in 0..4
                            {
                                let screen = barycentric(a, b, Vec2 { x: quad_x, y: quad_y });
                                let rhw = rhw0 * screen.x + rhw1 * screen.y + rhw2 * screen.z;
                                let w = 1. / rhw;
                                screens[i] = screen;
                                ws[i] = w;

                                let xx = quad_x as i32;
                                let yy = quad_y as i32;

                                if is_between(xx, min.x, max.x) && is_between(yy, min.y, max.y)
                                {
                                    //三角形重心坐标出现负值，证明在三角形外
                                    if screen.x < 0. || screen.y < 0. || screen.z < 0.
                                    {
            
                                    }
                                    else
                                    {
                                        let depth = if this.depth_value == GLFunction::Reciprocal_W
                                        { rhw }
                                        else
                                        { (z0 * screen.x + z1 * screen.y + z2 * screen.z) * w };
                                        zs[i] = depth;
            
                                        if !this.alpha_test
                                        {
                                            if let Some(depth_test_failed) = this.do_stencil_depth_test(xx, yy, i as i32, &mut valid, depth, fb)
                                            {
                                                some_test_failed = depth_test_failed;
                                            }
                                            else
                                            {
                                                some_test_failed = true;
                                            }
                                        }
                                        else
                                        {
                                            valid |= 1 << i;
                                        }
            
                                        inside = true;
                                    }
                                }
            
                                //0-0 1-1 2-0 3-1
                                quad_x += if i % 2 == 0 { 1. } else { -1. };
                                //0-0 1-0 2-1 3-1
                                quad_y += if i % 2 == 0 { 0. } else {  1. };
                            }

                            if valid != 0
                            {
                                for i in 0..varyings.len()
                                {
                                    varyings[i] = (v0 * screens[i].x + v1 * screens[i].y + v2 * screens[i].z) * ws[i];
                                    shader.sample(&varyings[i]);
                                }

                                shader.compute_level(this.anisotropic_filter as i32);

                                let mut xx = x;
                                let mut yy = y;

                                for i in 0..4
                                {
                                    if valid & (1 << i) != 0
                                    {
                                        let color = shader.fragment(&varyings[i], IVec2::new(xx, yy));

                                        if this.alpha_test
                                        {
                                            if compare_value(this.alpha_func, this.alpha_ref, color.a)
                                            {
                                                match this.do_stencil_depth_test(xx, yy, i as i32, &mut valid, zs[i], fb)
                                                {
                                                    //模板或者深度测试不通过
                                                    None |
                                                    Some(true) =>
                                                    {
                                                        //0-0 1-1 2-0 3-1
                                                        xx += if i % 2 == 0 { 1 } else { -1 };
                                                        //0-0 1-0 2-1 3-1
                                                        yy += if i % 2 == 0 { 0 } else {  1 };
                                                        continue;
                                                    }
            
                                                    _ => {}
                                                }
                                            }
                                            else
                                            {
                                                //0-0 1-1 2-0 3-1
                                                xx += if i % 2 == 0 { 1 } else { -1 };
                                                //0-0 1-0 2-1 3-1
                                                yy += if i % 2 == 0 { 0 } else {  1 };
                                                continue;
                                            }
                                        }

                                        let keep_one: u32 = fb.get_color(xx, yy).into();
            
                                        if this.blend
                                        {
                                            let color: u32 = this.do_blend_color(color, fb.get_color(xx, yy)).into();
                                            
                                            fb.set_color(xx, yy, GLColor::from((color & this.color_mask) | (keep_one & !this.color_mask)));
                                        }
                                        else
                                        {
                                            let color: u32 = color.into();
                                            fb.set_color(xx, yy, GLColor::from((color & this.color_mask) | (keep_one & !this.color_mask)));
                                        }
                                    }

                                    //0-0 1-1 2-0 3-1
                                    xx += if i % 2 == 0 { 1 } else { -1 };
                                    //0-0 1-0 2-1 3-1
                                    yy += if i % 2 == 0 { 0 } else { 1 };
                                }
                            }
                            else if inside && !some_test_failed
                            {
                                break;
                            }

                            x += 2;
                        }

                        y += threads * 2;
                    }
                });
            }
        });
    }

    /// bool为true代表模板测试通过，深度测试失败
    fn do_stencil_depth_test(&self, xx: i32, yy: i32, i: i32, valid: &mut i32, depth: f32, fb: &mut GLFrameBuffer) -> Option<bool>
    {
        //模板测试
        if self.stencil_test
        {
            if !compare_value(self.stencil_func, fb.get_stencil(xx, yy) & self.stencil_test_mask,
            self.stencil_ref & self.stencil_test_mask)
            {
                do_stencil_op(self.stencil_fail_op, xx, yy, self.stencil_write_mask, self.stencil_ref, fb);
                return None;
            }
        }

        //深度测试
        if self.depth_test
        {
            if compare_value(self.depth_func, fb.get_depth(xx, yy), depth)
            {
                if self.depth_mask != 0 { fb.set_depth(xx, yy, depth); }
                if self.stencil_test { do_stencil_op(self.all_pass_op, xx, yy, self.stencil_write_mask, self.stencil_ref, fb); }
                *valid |= 1 << i;
            }
            else
            {
                if self.stencil_test { do_stencil_op(self.depth_fail_op, xx, yy, self.stencil_write_mask, self.stencil_ref, fb); }
                return Some(true);
            }
        }
        else
        {
            if self.stencil_test { do_stencil_op(self.all_pass_op, xx, yy, self.stencil_write_mask, self.stencil_ref, fb); }
            *valid |= 1 << i;
        }

        return Some(false);
    }
}

/// 计算重心坐标
#[inline(always)]
fn barycentric(mut a: Vec3A, mut b: Vec3A, p: Vec2) -> Vec3A
{
    a.z -= p.x;
    b.z -= p.y;

    let u = a.cross(b);

    let z = 1. / u.z;
    Vec3A::new(1. - (u.x + u.y) * z, u.y * z, u.x * z)
}

pub fn compare_value<T: PartialOrd>(func: GLCompareFunc, old: T, new: T) -> bool
{
    match new.partial_cmp(&old).unwrap()
    {
        std::cmp::Ordering::Less    => (func & GLCompareFunc::Less) as i32    != 0,
        std::cmp::Ordering::Equal   => (func & GLCompareFunc::Equal) as i32   != 0,
        std::cmp::Ordering::Greater => (func & GLCompareFunc::Greater) as i32 != 0,   
    }
}

fn do_stencil_op(op: GLStencilOp, x: i32, y: i32, write_mask: u8, ref_value: u8, fb: &mut GLFrameBuffer)
{
    let keep_one = fb.get_stencil(x, y) & !write_mask;

    match op
    {
        GLStencilOp::Keep => (),

        GLStencilOp::Zero =>
        {
            fb.set_stencil(x, y, 0 | keep_one);
        }

        GLStencilOp::Replace =>
        {
            fb.set_stencil(x, y, (ref_value & write_mask) | keep_one);
        }

        GLStencilOp::Increase =>
        {
            fb.set_stencil(x, y, (fb.get_stencil(x, y).saturating_add(1) & write_mask) | keep_one);
        }

        GLStencilOp::IncreaseWrap =>
        {
            fb.set_stencil(x, y, (fb.get_stencil(x, y).wrapping_add(1) & write_mask) | keep_one);
        }

        GLStencilOp::Decrease =>
        {
            fb.set_stencil(x, y, (fb.get_stencil(x, y).saturating_sub(1) & write_mask) | keep_one);
        }

        GLStencilOp::DecreaseWrap =>
        {
            fb.set_stencil(x, y, (fb.get_stencil(x, y).wrapping_sub(1) & write_mask) | keep_one);
        }

        GLStencilOp::Invert =>
        {
            fb.set_stencil(x, y, (!fb.get_stencil(x, y) & write_mask) | keep_one);
        }
    }
}

fn _do_stencil_op_empty(_: GLStencilOp, _: i32, _: i32, _: u8, _: u8, _: &mut GLFrameBuffer)
{

}