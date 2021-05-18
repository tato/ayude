mod error;
pub use error::AyudeError;

pub mod graphics;

pub mod catalog;
pub use catalog::Catalog;
use catalog::Id;

pub mod import_gltf;

pub struct Entity {
    pub children: Vec<Id<Entity>>,
    pub parent: Option<Id<Entity>>,
    pub mesh: Option<Id<graphics::Mesh>>,
    pub transform: [[f32; 4]; 4],
    pub skin: Option<Id<Skin>>,
}

pub struct Skin {
    pub joints: Vec<Id<Entity>>,
}