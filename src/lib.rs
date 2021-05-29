mod error;
pub use error::AyudeError;

pub mod graphics;

pub mod catalog;
pub use catalog::Catalog;
use glam::Mat4;
use graphics::GraphicsContext;
use smallvec::SmallVec;
use transform::Transform;

pub mod camera;
pub mod import_gltf;
pub mod transform;

#[derive(Debug)]
pub struct Scene {
    pub nodes: Vec<Node>,
    pub root_nodes: SmallVec<[u16; 4]>,
    pub transform: Transform,
}

impl Scene {
    pub fn render(
        &self,
        gfx: &GraphicsContext,
        perspective: &Mat4,
        view: &Mat4,
        frame: &wgpu::SwapChainFrame,
        encoder: &mut wgpu::CommandEncoder,
    ) {
        let base_transform = &self.transform;
        for node in &self.nodes {
            if node.meshes.is_empty() {
                continue;
            }

            let transform = {
                let mut current = node;
                let mut transform = node.transform.mat4().clone();
                'transform: loop {
                    current = match current.parent {
                        Some(index) => &self.nodes[usize::from(index)],
                        None => break 'transform,
                    };

                    transform = transform.mul_mat4(current.transform.mat4());
                }
                Transform::from(transform)
            };

            for mesh in &node.meshes {
                let base_transform = base_transform.mat4().clone();
                let mesh_transform = transform.mat4().clone();
                let model = mesh_transform * base_transform;

                gfx.render_mesh(mesh, model, frame, encoder);
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct Node {
    pub parent: Option<u16>,
    pub children: SmallVec<[u16; 4]>,
    pub transform: Transform,
    pub meshes: Vec<graphics::Mesh>,
    pub skin: Option<Skin>,
}

#[derive(Debug, Clone)]
pub struct Skin {
    pub joints: SmallVec<[u16; 4]>,
    pub skeleton: Option<u16>,
}
