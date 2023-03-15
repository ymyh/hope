use glam::{Vec4, Vec2};

use super::glColor::GLColor;

pub struct GLTexture
{
    pub(crate) width: usize,
    pub(crate) height: usize,
    pub(crate) data: Vec<u8>,

    mipmaps: Vec<GLTexture>,
}

impl GLTexture
{
    pub fn from_bytes(data: &[u8], width: usize, height: usize) -> Option<Self>
    {
        if data.len() % 4 != 0
        {
            return None;
        }

        let data = Vec::from(data);
        
        Some(Self {
            width,
            height,
            data,

            mipmaps: Vec::new(),
        })
    }

    /// 只支持二次幂长宽的纹理，否则不会做任何事
    pub fn create_mipmap(&mut self, level: Option<u32>) -> bool
    {
        if self.width & (self.width - 1) != 0 || self.height & (self.height - 1) != 0
        {
            return false;
        }

        let level = level.unwrap_or(u32::MAX);
        self.create_mipmap_inner(self.width, self.height, level);

        return true;
    }

    fn create_mipmap_inner(&mut self, mut width: usize, mut height: usize, level: u32)
    {
        if (width == 1 && height == 1) || level == 0
        {
            return;
        }

        let last_mipmap = &self.mipmaps.last().unwrap_or(self).data;
        let colors = unsafe { std::slice::from_raw_parts(last_mipmap.as_ptr() as *const GLColor, last_mipmap.len() / 4) };

        let mut result: Vec<GLColor> = Vec::new();

        if width > 1 && height > 1
        {
            let mut i = 0;

            while i < height
            {
                let mut j = 0;

                while j < width
                {
                    let a: Vec4 = colors[(i + 0) * width + j + 0].into();
                    let b: Vec4 = colors[(i + 0) * width + j + 1].into();
                    let c: Vec4 = colors[(i + 1) * width + j + 0].into();
                    let d: Vec4 = colors[(i + 1) * width + j + 1].into();

                    result.push(((a + b + c + d) * 0.25).into());

                    j += 2;
                }

                i += 2;
            }
        }
        else
        {
            let mut i = 0;
            let limit = usize::max(width, height);

            while i < limit
            {
                let a: Vec4 = colors[i + 0].into();
                let b: Vec4 = colors[i + 1].into();

                result.push(((a + b) * 0.5).into());

                i += 2;
            }   
        }

        if width > 1 { width /= 2; }
        if height > 1 { height /= 2; }

        let mut data = vec![0u8; width * height * 4];
        data.clone_from_slice(unsafe { std::slice::from_raw_parts(result.as_ptr() as *const u8, width * height * 4) });

        self.mipmaps.push(Self {width, height, data, mipmaps: Vec::new()});

        self.create_mipmap_inner(width, height, level - 1);
    }

    pub fn set_mipmap(&mut self, mipmaps: Vec<GLTexture>)
    {
        self.mipmaps = mipmaps;
    }

    pub fn get_mipmap(&self, level: f32) -> &GLTexture
    {
        if level <= 0. || self.mipmaps.len() == 0
        {
            self
        }
        else
        {
            &self.mipmaps[usize::min(level as usize, self.mipmaps.len()) - 1]
        }
    }

    pub fn compute_st(&self, uv: Vec2) -> Vec2
    {
        Vec2::new(uv.x * (self.width as f32 - 1.), uv.y * (self.height as f32 - 1.))
    }

    #[unchecked::unchecked]
    pub fn get_value(&self, st: Vec2) -> GLColor
    {
        let ptr = (&self.data[usize::min(self.data.len() - 1, (st.y as usize * self.width + st.x as usize) * 4)]) as *const _ as *const GLColor;
        unsafe { *ptr }
    }
}