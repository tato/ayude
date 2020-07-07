use crate::*;

fn load_blend(file_name: &str) -> Result<gltf::UnloadedScene, AyudeError> {
    let blend = ::blend::Blend::from_path(file_name);

    

    todo!()
}