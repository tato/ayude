use crate::*;
use std::{io::Read, sync::mpsc};

pub type TextureId = usize;

struct ImageLoadedMessage {
    id: TextureId,
    bytes: Vec<u8>,
    width: u32,
    height: u32,
}

pub struct TextureRepository {
    executor: futures::executor::ThreadPool,
    image_loaded_sender: mpsc::Sender<ImageLoadedMessage>,
    image_loaded_receiver: mpsc::Receiver<ImageLoadedMessage>,

    textures: Vec<Option<u32>>,
    placeholder: u32,
}

impl TextureRepository {
    pub fn new() -> Self {

        let executor = futures::executor::ThreadPool::new().unwrap();
        let (image_loaded_sender, image_loaded_receiver) = mpsc::channel();

        let textures = vec![ ];
        let placeholder = unsafe {
            let (bytes, width, height) = futures::executor::block_on(load_texture_from_image_in_memory(include_bytes!("placeholder.png"))).unwrap();
            opengl::texture_from_data(&bytes, width as i32, height as i32)
        };

        Self{ executor, image_loaded_sender, image_loaded_receiver, textures, placeholder }
    }

    pub fn _get(&self, id: TextureId) -> &u32 {
        self.textures[id].as_ref().unwrap_or(&self.placeholder)
    }

    pub fn get_or_placeholder(&self, maybe_id: Option<TextureId>) -> &i32 /*glium::Texture2d*/ {
        // match maybe_id {
        //     None => &self.placeholder,
        //     Some(id) => &self.textures[id].as_ref().unwrap_or(&self.placeholder),
        // }
        todo!()
    }

    // pub fn _load_from_bytes(&mut self, source: Vec<u8>) -> TextureId {
    //     let result = self.textures.len();
    //     self.textures.push(None);
        
    //     let image_loaded_sender = self.image_loaded_sender.clone();
    //     self.executor.spawn_ok(async move {
    //         if let Some((bytes, width, height)) = load_texture_from_image_in_memory(&source).await {
    //             image_loaded_sender.send(ImageLoadedMessage{ id: result, bytes, width, height }).unwrap();
    //         }
    //     });

    //     result
    // }

    pub fn load_from_file_name(&mut self, file_name: String) -> TextureId {
        // let result = self.textures.len();
        // self.textures.push(None);

        // let image_loaded_sender = self.image_loaded_sender.clone();
        // self.executor.spawn_ok(async move {
        //     if let Some((bytes, width, height)) = load_texture_from_file_name(file_name).await {
        //         image_loaded_sender.send(ImageLoadedMessage{ id: result, bytes, width, height }).unwrap();
        //     }
        // });

        // result
        todo!()
    }

    pub fn poll_textures(&mut self, display: &i32 /*glium::Display*/) {
        // if let Ok(message) = self.image_loaded_receiver.try_recv() {
        //     let raw_image = glium::texture::RawImage2d::from_raw_rgba(message.bytes, (message.width, message.height));
        //     let texture = glium::Texture2d::new(display, raw_image).ok();
        //     self.textures[message.id] = texture;
        // }
        todo!()
    }
}

async fn load_texture_from_file_name(file_name: String) -> Option<(Vec<u8>, u32, u32)> {
    let source = {
        let mut source = Vec::new();
        std::fs::File::open(&file_name).ok()?.read_to_end(&mut source).ok()?;
        source
    };
    load_texture_from_image_in_memory(&source).await
}

async fn load_texture_from_image_in_memory(input: &[u8]) -> Option<(Vec<u8>, u32, u32)> {

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