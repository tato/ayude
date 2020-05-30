mod geometry;
pub use geometry::Geometry;

mod texture;
pub use texture::Texture;

mod shader;
pub use shader::Shader;

mod error;
pub use error::GraphicsError;

pub fn start_frame(color: [f32; 3]) {
    unsafe {
        gl::ClearColor(color[0], color[1], color[2], 1.0);
        gl::ClearDepth(1.0);
        gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
    }
}