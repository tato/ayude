use std::u8;

use std::cell::RefCell;
use rusttype::{Scale, gpu_cache::Cache, point};

use super::Texture;

pub struct Font {
    font: rusttype::Font<'static>,
    cache: Cache<'static>,
    texture: Texture,
}

impl Font {
    pub fn from_bytes(bytes: Vec<u8>) -> Result<Self, FontError> {
        let font = rusttype::Font::try_from_vec(bytes).ok_or(FontError::RustTypeError)?;
        let (cache_width, cache_height) = (512, 512);
        let cache = Cache::builder()
            .dimensions(cache_width, cache_height)
            .build();
        let cache = cache.into();
        let texture = todo!("empty 512x512 texture");
        Ok(Self {
            font,
            cache,
            texture,
        })
    }

    pub fn render(&mut self, text: &str, scale: f32) -> Vec<()> {

        let scale = Scale::uniform(scale);

        let v_metrics = self.font.v_metrics(scale);
        let offset = rusttype::point(0.0, v_metrics.ascent);

        let mut caret = point(0.0, v_metrics.ascent);
        let mut last_glyph_id = None;

        for c in text.chars() {
            let base_glyph = self.font.glyph(c);

            if let Some(id) = last_glyph_id.take() {
                caret.x += self.font.pair_kerning(scale, id, base_glyph.id());
            }
            last_glyph_id = Some(base_glyph.id());

            let mut glyph = base_glyph.scaled(scale).positioned(caret);
            caret.x += glyph.unpositioned().h_metrics().advance_width;
            
            self.cache.queue_glyph(0, glyph.clone());
        }

        self.cache.cache_queued(|rect, data| {
            self.texture.write();
            // cache_tex.main_level().write(
            //     glium::Rect {
            //         left: rect.min.x,
            //         bottom: rect.min.y,
            //         width: rect.width(),
            //         height: rect.height(),
            //     },
            //     glium::texture::RawImage2d {
            //         data: Cow::Borrowed(data),
            //         width: rect.width(),
            //         height: rect.height(),
            //         format: glium::texture::ClientFormat::U8,
            //     },
            // );
        });

        
        // let test_font_texture = {

        //     let height: f32 = 12.4;
        //     let pixel_height = height.ceil() as usize;

        //     let scale = Scale {
        //         x: height * 2.0,
        //         y: height,
        //     };

        //     let v_metrics = font.v_metrics(scale);
        //     let offset = rusttype::point(0.0, v_metrics.ascent);

        //     let glyphs: Vec<_> = font.layout("RIGHT NOW.", scale, offset).collect();

        //     let width = glyphs
        //         .iter()
        //         .rev()
        //         .map(|g| g.position().x as f32 + g.unpositioned().h_metrics().advance_width)
        //         .next()
        //         .unwrap_or(0.0)
        //         .ceil() as usize;

        //     let mut pixel_data = vec![0u8; width * pixel_height * 4];
        //     for g in glyphs {
        //         if let Some(bb) = g.pixel_bounding_box() {
        //             g.draw(|x, y, v| {
        //                 let gray = (v * 255.5) as u8;
        //                 let x = x as i32 + bb.min.x ;
        //                 let y = y as i32 + bb.min.y;
        //                 if x >= 0 && x < width as i32 && y >= 0 && y < pixel_height as i32 {
        //                     let i = (y as usize * width + x as usize) * 4;
        //                     pixel_data[i] = gray;
        //                     pixel_data[i + 1] = gray;
        //                     pixel_data[i + 2] = gray;
        //                     pixel_data[i + 3] = 255;
        //                 }
        //             });
        //         }
        //     }

        //     graphics::Texture::builder(
        //         &pixel_data,
        //         width as u16,
        //         pixel_height as u16,
        //         graphics::texture::TextureFormat::RGBA,
        //     )
        //     .build()
        // };
        
        vec![]
    }
}

// https://gitlab.redox-os.org/redox-os/rusttype/-/blob/master/dev/examples/gpu_cache.rs




#[derive(thiserror::Error, Debug)]
pub enum FontError {
    #[error("unable to parse font")]
    RustTypeError,
}
