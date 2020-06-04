use std::{io::Read, rc};

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
    pub fn from_file_name(file_name: &str) -> Option<Self> {
        let (bytes, width, height) = futures::executor::block_on(load_texture_from_file_name(file_name))?;
        Some(Self::from_rgba(&bytes, width as i32, height as i32))
    }
}

// impl Drop for Texture {
//     fn drop(&mut self) {
//         if 1 == rc::Rc::strong_count(&self.id) {
//             unsafe { gl::DeleteTextures(1, &self.id); }
//         }
//     }
// }

// pub struct TextureRepository {
//     executor: futures::executor::ThreadPool,
//     image_loaded_sender: mpsc::Sender<ImageLoadedMessage>,
//     image_loaded_receiver: mpsc::Receiver<ImageLoadedMessage>,
//
//     textures: Vec<>,
//     placeholder: u32,
// }
//
// struct ImageLoadedMessage {
//     id: TextureId,
//     bytes: Vec<u8>,
//     width: u32,
//     height: u32,
// }
//
// impl TextureRepository {
//     pub fn new() -> Self {
//         let executor = futures::executor::ThreadPool::new().unwrap();
//         let (image_loaded_sender, image_loaded_receiver) = mpsc::channel();
//
//         let textures = vec![ ];
//         let placeholder = {
//             let (bytes, width, height) = futures::executor::block_on({
//                 load_texture_from_image_in_memory(include_bytes!("../resources/placeholder.png"))
//             }).unwrap();
//             create_texture_from_data(&bytes, width as i32, height as i32)
//         };
//
//         Self{ executor, image_loaded_sender, image_loaded_receiver, textures, placeholder }
//     }
//
//     // pub fn _load_from_bytes(&mut self, source: Vec<u8>) -> Texture {
//     //     let result = self.textures.len();
//     //     self.textures.push(None);
//     //
//     //     let image_loaded_sender = self.image_loaded_sender.clone();
//     //     self.executor.spawn_ok(async move {
//     //         if let Some((bytes, width, height)) = load_texture_from_image_in_memory(&source).await {
//     //             image_loaded_sender.send(ImageLoadedMessage{ id: result, bytes, width, height }).unwrap();
//     //         }
//     //     });
//     //
//     //     result
//     // }
//
//     // todo! change `file_name` from `String` to `&str`
//     pub fn create_from_file_name(&mut self, file_name: String) -> rc::Rc<Texture> {
//         let mut texture_id = 0;
//         gl::GenTextures(1, &mut texture_id);
//
//         let result = self.textures.len();
//         self.textures.push(None);
//
//         let image_loaded_sender = self.image_loaded_sender.clone();
//         self.executor.spawn_ok(async move {
//             if let Some((bytes, width, height)) = load_texture_from_file_name(file_name).await {
//                 image_loaded_sender.send(ImageLoadedMessage{ id: result, bytes, width, height }).unwrap();
//             }
//         });
//
//         result
//     }
//
//     pub fn poll_textures(&mut self) {
//         if let Ok(message) = self.image_loaded_receiver.try_recv() {
//             let texture = create_texture_from_data(&message.bytes, message.width as i32, message.height as i32);
//             self.textures[message.id] = texture;
//         }
//     }
// }
//

pub async fn load_texture_from_file_name(file_name: &str) -> Option<(Vec<u8>, u32, u32)> {
    let start = std::time::Instant::now();
    let source = {
        let mut source = Vec::new();
        std::fs::File::open(file_name).ok()?.read_to_end(&mut source).ok()?;
        source
    };
    let result = load_texture_from_image_in_memory(&source).await;
    println!("{}: {:?}", file_name, start.elapsed());
    result
}

pub async fn load_texture_from_image_in_memory(input: &[u8]) -> Option<(Vec<u8>, u32, u32)> {

    let mut width: i32 = 0;
    let mut height: i32 = 0;
    let mut channels: i32 = 0;

    unsafe {
        let bytes = stb_image::stb_image::bindgen::stbi_load_from_memory(
            input.as_ptr(),
            input.len() as i32,
            &mut width as *mut i32,
            &mut height as *mut i32,
            &mut channels as *mut i32,
            4
        );

        if bytes.is_null() {
            let _reason = std::ffi::CStr::from_ptr(stb_image::stb_image::bindgen::stbi_failure_reason());
            None
        } else {
            let bytes_length = (width*height*4) as usize;
            // i think `Vec::from_raw_parts` could leak a little bit here (`malloc` metadata or something)
            let owned = Vec::from_raw_parts(bytes, bytes_length, bytes_length);
            Some((owned, width as u32, height as u32))
        }
    }
}
//
// fn create_texture_from_data(rgba: &[u8], width: i32, height: i32) -> u32 {
//     let mut texture_id = 0u32;
//
//     unsafe {
//         gl::GenTextures(1, &mut texture_id);
//         gl::BindTexture(gl::TEXTURE_2D, texture_id);
//
//         gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::REPEAT as i32);
//         gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::REPEAT as i32);
//         gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
//         gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
//
//         gl::TexImage2D(gl::TEXTURE_2D, 0, gl::RGBA as i32, width as i32, height as i32, 0, gl::RGBA, gl::UNSIGNED_BYTE, rgba.as_ptr() as *const std::ffi::c_void);
//     }
//
//     texture_id
// }