use std::rc;

#[derive(Clone)]
pub struct Texture {
    pub id: rc::Rc<u32>,
}

impl Texture {
    pub fn empty() -> Self {
        Texture{ id: 0.into() } // todo! this is debug only!!!!
    }
    pub fn from_rgba(rgba: &[u8], width: i32, height: i32) -> Self {
        let mut id = 0u32;

        unsafe {
            gl::GenTextures(1, &mut id);
            gl::BindTexture(gl::TEXTURE_2D, id);

            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::REPEAT as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::REPEAT as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);

            gl::TexImage2D(gl::TEXTURE_2D, 0, gl::RGBA as i32, width as i32, height as i32, 0, gl::RGBA, gl::UNSIGNED_BYTE, rgba.as_ptr() as *const std::ffi::c_void);
        }

        Texture{ id: id.into() }
    }
}

// todo!
// impl Drop for Texture {
//     fn drop(&mut self) {
//         if 1 == rc::Rc::strong_count(&self.id) {
//             unsafe { gl::DeleteTextures(1, &self.id); }
//         }
//     }
// }