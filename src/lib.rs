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
    pub fn render<'scene: 'pass, 'pass>(
        &'scene self,
        pass: &'pass mut graphics::Pass<'scene, 'scene>,
        perspective: Mat4,
        view: Mat4,
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

                    transform = transform * current.transform.mat4();
                }
                Transform::from(transform)
            };

            for (mesh, ub, material) in &node.meshes {
                let base_transform = base_transform.mat4();
                let mesh_transform = transform.mat4();
                let model = mesh_transform * base_transform;

                pass.render_mesh(mesh, ub, material, perspective, view, model);
            }
        }
    }

    pub fn duplicate(&self, graphics: &GraphicsContext) -> Self {
        Self {
            nodes: self.nodes.iter().map(|it| it.duplicate(graphics)).collect(),
            root_nodes: self.root_nodes.clone(),
            transform: self.transform.clone(),
        }
    }
}

#[derive(Debug)]
pub struct Node {
    pub parent: Option<u16>,
    pub children: SmallVec<[u16; 4]>,
    pub transform: Transform,
    pub meshes: Vec<(graphics::Mesh, graphics::UniformBuffer, graphics::Material)>,
    pub skin: Option<Skin>,
    pub name: Option<String>,
}

impl Node {
    pub fn duplicate(&self, graphics: &GraphicsContext) -> Self {
        Self {
            parent: self.parent.clone(),
            children: self.children.clone(),
            transform: self.transform.clone(),
            meshes: self
                .meshes
                .iter()
                .map(|(mesh, _, mat)| {
                    (mesh.clone(), graphics.create_uniform_buffer(), mat.clone())
                })
                .collect(),
            skin: self.skin.clone(),
            name: self.name.clone(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Skin {
    pub joints: SmallVec<[u16; 4]>,
    pub skeleton: Option<u16>,
}
