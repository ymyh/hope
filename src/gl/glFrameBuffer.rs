use crate::gl::glColor::GLColor;

pub struct GLFrameBuffer
{
    width: usize,
    height: usize,

    color_buffer: Vec<GLColor>,
    depth_buffer: Vec<f32>,
    stencil_buffer: Vec<u8>,
}

impl GLFrameBuffer
{
    pub fn new(width: usize, height: usize) -> Self
    {
        return Self {
            width,
            height,

            color_buffer: Vec::new(),
            depth_buffer: Vec::new(),
            stencil_buffer: Vec::new(),
        }
    }

    pub fn attach_color(&mut self)
    {
        self.color_buffer.resize(self.width * self.height, GLColor::ONE);
    }

    pub fn attach_depth(&mut self)
    {
        self.depth_buffer.resize(self.width * self.height, 1.0);
    }

    pub fn attach_stencil(&mut self)
    {
        self.stencil_buffer.resize(self.width * self.height, 0x00);
    }

    pub fn attach_all(&mut self)
    {
        self.attach_color();
        self.attach_depth();
        self.attach_stencil();
    }

    pub fn get_width(&self) -> usize
    {
        self.width
    }

    pub fn get_height(&self) -> usize
    {
        self.height
    }

    pub fn get_color_buffer(&self) -> &[u8]
    {
        unsafe
        {
            &*std::ptr::slice_from_raw_parts(self.color_buffer.as_ptr() as *const u8, 
            self.color_buffer.len() * 4)
        }
    }

    pub fn get_depth_buffer(&self) -> &[f32]
    {
        &self.depth_buffer
    }

    pub fn get_stencil_buffer(&self) -> &[u8]
    {
        &self.stencil_buffer
    }

    pub fn take_color_buffer(&mut self) -> Vec<GLColor>
    {
        let result = std::mem::take(&mut self.color_buffer);
        self.attach_color();

        result
    }

    #[unchecked::unchecked]
    pub fn get_color(&self, x: i32, y: i32) -> GLColor
    {
        self.color_buffer[y as usize * self.width + x as usize]
    }

    #[unchecked::unchecked]
    pub fn get_depth(&self, x: i32, y: i32) -> f32
    {
        self.depth_buffer[y as usize * self.width + x as usize]
    }

    #[unchecked::unchecked]
    pub fn get_stencil(&self, x: i32, y: i32) -> u8
    {
        self.stencil_buffer[y as usize * self.width + x as usize]
    }

    #[unchecked::unchecked]
    pub fn set_color(&mut self, x: i32, y: i32, color: GLColor)
    {
        self.color_buffer[y as usize * self.width + x as usize] = color;
    }

    #[unchecked::unchecked]
    pub fn set_depth(&mut self, x: i32, y: i32, depth: f32)
    {
        self.depth_buffer[y as usize * self.width + x as usize] = depth;
    }

    #[unchecked::unchecked]
    pub fn set_stencil(&mut self, x: i32, y: i32, stencil: u8)
    {
        self.stencil_buffer[y as usize * self.width + x as usize] = stencil;
    }

    pub fn set_color_buffer(&mut self, buffer: Vec<GLColor>)
    {
        self.color_buffer = buffer;
    }

    pub(crate) fn do_clear_color(&mut self, color: GLColor)
    {
        if color.r == color.g &&
        color.r == color.b &&
        color.r == color.a
        {
            unsafe
            {
                std::ptr::write_bytes(self.color_buffer.as_mut_ptr(), color.r,
                    self.color_buffer.len());
            }
        }
        else
        {
            let ptr = self.color_buffer.as_mut_ptr() as *mut u32;
            let end_idx = self.color_buffer.len();
            let mut current_idx = 4;

            unsafe
            {
                *ptr = color.into();

                while current_idx << 1 <= end_idx
                {
                    self.color_buffer.copy_within(0..current_idx, current_idx);
                    current_idx <<= 1;
                }

                if current_idx != end_idx
                {
                    self.color_buffer.copy_within(0..(end_idx - current_idx), current_idx);
                }
            }
        }
    }

    pub(crate) fn do_clear_depth(&mut self, depth: f32)
    {
        let ptr = self.depth_buffer.as_mut_ptr() as *mut f32;
        let end_idx = self.depth_buffer.len();
        let mut current_idx = 1;

        unsafe
        {
            *ptr = depth;

            while current_idx << 1 <= end_idx
            {
                self.depth_buffer.copy_within(0..current_idx, current_idx);
                current_idx <<= 1;
            }

            if current_idx != end_idx
            {
                self.depth_buffer.copy_within(0..(end_idx - current_idx), current_idx);
            }
        }
    }

    pub(crate) fn do_clear_stencil(&mut self, stencil: u8)
    {
        unsafe
        {
            std::ptr::write_bytes(self.stencil_buffer.as_mut_ptr(), stencil,
                self.stencil_buffer.len());
        }
    }
}