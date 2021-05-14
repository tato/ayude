mod error;
pub use error::AyudeError;

pub mod graphics;

pub mod catalog;
use catalog::Id;
pub use catalog::Catalog;

pub mod import_gltf;

pub struct Entity {
    pub children: Vec<Id<Entity>>,
    pub parent: Option<Id<Entity>>,
    pub mesh: Option<Id<graphics::Mesh>>,
    pub transform: [[f32; 4]; 4],
}