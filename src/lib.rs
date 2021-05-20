mod error;
pub use error::AyudeError;

pub mod graphics;

pub mod catalog;
pub use catalog::Catalog;

pub mod import_gltf;

pub struct Entity {
    pub meshes: Vec<graphics::Mesh>,
    pub mesh_transforms: Vec<Transform>,
    pub transform: Transform,
    pub skin: Option<Skin>,
}

#[derive(Debug, Clone)]
pub struct Skin {
    pub joints: Vec<Joint>,
}

#[derive(Debug, Clone)]
pub struct Joint {
    pub transform: Transform,
    pub children: [usize; 4],
}

#[derive(Clone, Debug)]
pub struct Transform([[f32; 4]; 4]);

impl Transform {
    pub fn new(mat: [[f32; 4]; 4]) -> Transform {
        Transform(mat)
    }
    pub fn mat4(&self) -> &[[f32; 4]; 4] {
        &self.0
    }
}