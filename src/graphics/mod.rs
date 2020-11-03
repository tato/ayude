mod mesh;
pub use mesh::{Mesh, Primitive};

pub mod texture;
pub use texture::Texture;

mod shader;
pub use shader::Shader;

pub struct Material {
    pub normal: Option<crate::catalog::Id<Texture>>,
    pub diffuse: Option<crate::catalog::Id<Texture>>,
    pub base_diffuse_color: [f32; 4],
}

pub struct Frame {
    viewport: (i32, i32, i32, i32),
}

impl Frame {
    pub fn start(clear_color: [f32; 3], window_dimensions: (i32, i32)) -> Frame {
        unsafe {
            gl::ClearColor(clear_color[0], clear_color[1], clear_color[2], 1.0);
            gl::ClearDepth(1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        }
        Frame {
            viewport: (0, 0, window_dimensions.0, window_dimensions.1),
        }
    }

    pub fn render(&self, primitive: &mesh::Primitive, shader: &Shader) {
        unsafe {
            // gl::Enable(gl::BLEND);
            // gl::BlendEquation(gl::FUNC_ADD);
            // gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);

            gl::Enable(gl::DEPTH_TEST);
            gl::DepthFunc(gl::LEQUAL);
            gl::Disable(gl::CULL_FACE); // CullClockwise

            gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
            gl::Viewport(
                self.viewport.0,
                self.viewport.1,
                self.viewport.2,
                self.viewport.3,
            );

            shader.bind();
            gl::BindVertexArray(primitive.vao);
            gl::DrawElements(
                gl::TRIANGLES,
                primitive.element_count,
                gl::UNSIGNED_SHORT,
                std::ptr::null(),
            );
        }
    }
}
