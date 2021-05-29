use std::rc::Rc;


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextureFormat {
    RGB,
    RGBA,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextureWrap {
    ClampToEdge,
    MirroredRepeat,
    Repeat,
}

impl TextureWrap {
    pub fn into_gl(self) -> u32 {
        todo!()
        // match self {
        //     Self::ClampToEdge => gl::CLAMP_TO_EDGE,
        //     Self::MirroredRepeat => gl::MIRRORED_REPEAT,
        //     Self::Repeat => gl::REPEAT,
        // }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MinFilter {
    Nearest,
    Linear,
    NearestMipmapNearest,
    LinearMipmapNearest,
    NearestMipmapLinear,
    LinearMipmapLinear,
}

impl MinFilter {
    pub fn into_gl(self) -> u32 {
        todo!()
        // match self {
        //     Self::Nearest => gl::NEAREST,
        //     Self::Linear => gl::LINEAR,
        //     Self::NearestMipmapNearest => gl::NEAREST_MIPMAP_NEAREST,
        //     Self::LinearMipmapNearest => gl::LINEAR_MIPMAP_NEAREST,
        //     Self::NearestMipmapLinear => gl::NEAREST_MIPMAP_LINEAR,
        //     Self::LinearMipmapLinear => gl::LINEAR_MIPMAP_LINEAR,
        // }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MagFilter {
    Nearest,
    Linear,
}

impl MagFilter {
    pub fn into_gl(self) -> u32 {
        todo!()
        // match self {
        //     Self::Nearest => gl::NEAREST,
        //     Self::Linear => gl::LINEAR,
        // }
    }
}

pub struct TextureBuilder<'data> {
    data: &'data [u8],
    width: i32,
    height: i32,
    format: TextureFormat,
    wrap_s: TextureWrap,
    wrap_t: TextureWrap,
    min_filter: MinFilter,
    mag_filter: MagFilter,
}

impl<'data> TextureBuilder<'data> {
    pub fn build(self) -> Texture {
        let mut id = 0u32;

        todo!();
        // let format = match self.format {
        //     TextureFormat::RGB => gl::RGB,
        //     TextureFormat::RGBA => gl::RGBA,
        // };

        // unsafe {
        //     gl::GenTextures(1, &mut id);
        //     gl::BindTexture(gl::TEXTURE_2D, id);

        //     gl::TexParameteri(
        //         gl::TEXTURE_2D,
        //         gl::TEXTURE_WRAP_S,
        //         self.wrap_s.into_gl() as i32,
        //     );
        //     gl::TexParameteri(
        //         gl::TEXTURE_2D,
        //         gl::TEXTURE_WRAP_T,
        //         self.wrap_t.into_gl() as i32,
        //     );
        //     gl::TexParameteri(
        //         gl::TEXTURE_2D,
        //         gl::TEXTURE_MIN_FILTER,
        //         self.min_filter.into_gl() as i32,
        //     );
        //     gl::TexParameteri(
        //         gl::TEXTURE_2D,
        //         gl::TEXTURE_MAG_FILTER,
        //         self.mag_filter.into_gl() as i32,
        //     );

        //     gl::TexImage2D(
        //         gl::TEXTURE_2D,
        //         0,
        //         gl::COMPRESSED_RGBA as i32,
        //         self.width,
        //         self.height,
        //         0,
        //         format,
        //         gl::UNSIGNED_BYTE,
        //         self.data.as_ptr() as *const std::ffi::c_void,
        //     );
        // }

        Texture { id: id.into(), width: self.width, height: self.height }
    }

    pub fn wrap_s(mut self, wrap: TextureWrap) -> Self {
        self.wrap_s = wrap;
        self
    }
    pub fn wrap_t(mut self, wrap: TextureWrap) -> Self {
        self.wrap_t = wrap;
        self
    }
    pub fn min_filter(mut self, filter: MinFilter) -> Self {
        self.min_filter = filter;
        self
    }
    pub fn mag_filter(mut self, filter: MagFilter) -> Self {
        self.mag_filter = filter;
        self
    }
}
