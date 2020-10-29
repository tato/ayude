use crate::*;
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Document {
    extensions_used: Vec<String>,
    extensions_required: Vec<String>,
    pub accessors: Vec<Accesor>,
    pub animations: Vec<Animation>,
    pub asset: Asset,
    pub buffers: Vec<Buffer>,
    pub buffer_views: Vec<BufferView>,
    pub cameras: Vec<Camera>,
    #[serde(default)]
    pub images: Vec<Image>,
    pub materials: Vec<Material>,
    pub meshes: Vec<Mesh>,
    pub nodes: Vec<Node>,
    #[serde(default)]
    pub samplers: Vec<Sampler>,
    pub scene: Option<usize>,
    pub scenes: Vec<Scene>,
    pub skins: Vec<Skin>,
    #[serde(default)]
    pub textures: Vec<Texture>,
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Image {
    uri: Option<String>,
    mime_type: Option<String>,
    buffer_view: Option<usize>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Asset {
    generator: String,
    version: String,
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Node {
    pub camera: Option<usize>,
    #[serde(default)]
    pub children: Vec<usize>,
    pub skin: Option<usize>,
    pub matrix: Option<[f32; 16]>,
    pub mesh: Option<usize>,
    pub translation: Option<[f32; 3]>,
    pub rotation: Option<[f32; 4]>,
    pub scale: Option<[f32; 3]>,
    #[serde(default)]
    pub weights: Vec<usize>,
    pub name: String,
    // extensions: object
    // extras: object
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Buffer {
    byte_length: usize,
    uri: String,
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BufferView {
    buffer: usize,
    byte_length: usize,
    byte_offset: usize,
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Accesor {
    buffer_view: usize,
    component_type: usize,
    #[serde(alias = "type")]
    _type: String,
    count: usize,
    #[serde(default)]
    byte_offset: usize,
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Sampler {}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Animation {}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Camera {}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Skin {}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Texture {
    sampler: usize,
    source: usize,
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Material {
    alpha_mode: Option<String>,
    double_sided: Option<bool>,
    pbr_metallic_roughness: PbrMetallicRoughness,
    normal_texture: Option<TextureInfo>,
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PbrMetallicRoughness {
    base_color_factor: [f32; 4],
    base_color_texture: Option<TextureInfo>,
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TextureInfo {
    index: usize,
    scale: Option<f32>,
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Mesh {
    pub primitives: Vec<Primitive>,
    #[serde(default)]
    pub weights: Vec<i32>,
    pub name: String,
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Primitive {
    indices: usize,
    material: usize,
    mode: Option<usize>,
    attributes: HashMap<String, usize>,
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Scene {
    pub nodes: Vec<usize>,
}

pub struct GLTF {
    pub document: Document,
    gltf_base_folder: String,
}

pub fn load(file_name: &str) -> Result<GLTF, AyudeError> {
    let document: Document = serde_json::from_str(&std::fs::read_to_string(file_name)?)?;

    assert!(
        document
            .scene
            .map(|s| s < document.scenes.len())
            .unwrap_or(true),
        "default scene index (`scene`) field isn't a valid scene index (value = {:?})",
        document.scene
    );
    assert_eq!(
        document.extensions_required.len(),
        0,
        "extensions are required to load this gltf document. `extensionsRequired` = {:?}",
        document.extensions_required
    );

    let gltf_base_folder = file_name
        .rfind('/')
        .map(|idx| &file_name[0..idx + 1])
        .unwrap_or("")
        .to_string();

    Ok(GLTF {
        document,
        gltf_base_folder,
    })
}

// todo! delete
// pub fn load(file_name: &str) -> Result<UnloadedScene, AyudeError> {
//     let document: Document = serde_json::from_str(&std::fs::read_to_string(file_name)?)?;
//     assert!(document.scene.map(|s| s < document.scenes.len()).unwrap_or(true));

//     let gltf_base_folder = file_name.rfind('/')
//         .map(|idx| &file_name[0..idx+1])
//         .unwrap_or("");

//     let buffers: Vec<Vec<u8>> = document.buffers.iter().map(|b| {
//         let mut result = Vec::new();
//         std::fs::File::open(format!("{}{}", gltf_base_folder, b.uri))?.read_to_end(&mut result)?;
//         Ok(result)
//     }).collect::<Result<_, AyudeError>>()?;

//     let mut images = Vec::new();
//     let mut images_byte_buffer = Vec::new();
//     for image in &document.images {
//         let image_file_name = format!("{}{}", gltf_base_folder, image.uri);
//         let loaded = image::open(&image_file_name)?.into_rgba();
//         let width = loaded.width();
//         let height = loaded.height();
//         let bytes = image::DynamicImage::ImageRgba8(loaded).to_bytes();
//         images.push(UnloadedImage{ offset: images_byte_buffer.len(), size: bytes.len(), width, height });
//         images_byte_buffer.extend(bytes);
//     }

//     let mut nodes = Vec::new();
//     let mut geometries = Vec::new();

//     let y_up_to_z_up_transform = Mat4::from_cols_array(&[
//         0.0, 1.0, 0.0, 0.0,
//         0.0, 0.0, 1.0, 0.0,
//         1.0, 0.0, 0.0, 0.0,
//         0.0, 0.0, 0.0, 1.0f32,
//     ]);

//     let mut node_queue = Vec::new();
//     let mut transform_queue = Vec::new();

//     let default_scene_nodes = document.scenes[document.scene.unwrap_or(0)]
//         .nodes.iter()
//         .map(|index| &document.nodes[*index]);
//     node_queue.extend(default_scene_nodes);

//     while !node_queue.is_empty() {
//         let node = node_queue.pop().unwrap();
//         let parent_transform = transform_queue.pop().unwrap_or(Mat4::identity());

//         let mesh = match node.mesh {
//             Some(index) => &document.meshes[index],
//             None => continue,
//         };

//         let node_local_transform = {
//             if let Some(m) = node.matrix {
//                 Mat4::from_cols_array(&m)
//             } else {
//                 let t: Vec3 = node.translation.unwrap_or([0.0, 0.0, 0.0]).into();
//                 let r: Quat = node.rotation.unwrap_or([0.0, 0.0, 0.0, 1.0]).into();
//                 let s: Vec3 = node.scale.unwrap_or([1.0, 1.0, 1.0]).into();
//                 Mat4::from_translation(t) * Mat4::from_quat(r) * Mat4::from_scale(s)
//             }
//         };
//         let transform = parent_transform * node_local_transform;

//         for index in &node.children {
//             node_queue.push(&document.nodes[*index]);
//             transform_queue.push(transform);
//         }

//         for primitive in &mesh.primitives {
//             let positions: &[[f32; 3]] = {
//                 let accessor = &document.accessors[primitive.attributes["POSITION"]];
//                 debug_assert!(accessor.component_type == 5126);
//                 debug_assert!(accessor._type == "VEC3");
//                 let view = &document.buffer_views[accessor.buffer_view];
//                 let buffer = &buffers[view.buffer][view.byte_offset..(view.byte_offset+view.byte_length)];
//                 unsafe {
//                     let ptr = std::mem::transmute(buffer.as_ptr());
//                     std::slice::from_raw_parts(ptr, buffer.len() / 12)
//                 }
//             };
//             let normals: &[[f32; 3]] = {
//                 let accessor = &document.accessors[primitive.attributes["NORMAL"]];
//                 debug_assert!(accessor.component_type == 5126);
//                 debug_assert!(accessor._type == "VEC3");
//                 let view = &document.buffer_views[accessor.buffer_view];
//                 let buffer = &buffers[view.buffer][view.byte_offset..(view.byte_offset+view.byte_length)];
//                 unsafe {
//                     let ptr = std::mem::transmute(buffer.as_ptr());
//                     std::slice::from_raw_parts(ptr, buffer.len() / 12)
//                 }
//             };
//             let uvs: &[[f32; 2]] = {
//                 let accessor = &document.accessors[primitive.attributes["TEXCOORD_0"]];
//                 debug_assert!(accessor.component_type == 5126);
//                 debug_assert!(accessor._type == "VEC2");
//                 let view = &document.buffer_views[accessor.buffer_view];
//                 let buffer = &buffers[view.buffer][view.byte_offset..(view.byte_offset+view.byte_length)];
//                 unsafe {
//                     let ptr = std::mem::transmute(buffer.as_ptr());
//                     std::slice::from_raw_parts(ptr, buffer.len() / 8)
//                 }
//             };

//             let indices: &[u16] = {
//                 let accessor = &document.accessors[primitive.indices];
//                 debug_assert!(accessor.component_type == 5123, "accessor.componentType ({}) == 5123", accessor.component_type);
//                 debug_assert!(accessor._type == "SCALAR");
//                 let view = &document.buffer_views[accessor.buffer_view];
//                 let buffer = &buffers[view.buffer][view.byte_offset..(view.byte_offset+view.byte_length)];
//                 unsafe {
//                     let ptr = std::mem::transmute(buffer.as_ptr());
//                     std::slice::from_raw_parts(ptr, buffer.len() / 2)
//                 }
//             };

//             let material = &document.materials[primitive.material];
//             let diffuse = material.pbr_metallic_roughness.base_color_texture.as_ref().map(|info| {
//                 document.textures[info.index].source
//             });
//             let normal = material.normal_texture.as_ref().map(|info| {
//                 document.textures[info.index].source
//             });

//             let base_diffuse_color = material.pbr_metallic_roughness.base_color_factor;

//             let positions = positions.to_vec();
//             let normals = normals.to_vec();
//             let uvs = uvs.to_vec();
//             let indices = indices.to_vec();
//             let transform = (y_up_to_z_up_transform * transform).to_cols_array_2d();

//             let geometry = UnloadedGeometry{positions, normals, uvs, indices};
//             geometries.push(geometry);

//             let geometry_id = geometries.len() - 1;

//             nodes.push(UnloadedSceneNode{ geometry_index: geometry_id, transform, diffuse, normal, base_diffuse_color });
//         }
//     }

//     Ok(UnloadedScene{ nodes, images, geometries, images_byte_buffer })
// }

// #[derive(Debug, Serialize, Deserialize)]
// pub struct UnloadedSceneNode {
//     pub geometry_index: usize,

//     pub transform: [[f32; 4]; 4],

//     pub diffuse: Option<usize>,
//     pub normal: Option<usize>,

//     pub base_diffuse_color: [f32; 4],
// }
// #[derive(Serialize, Deserialize)]
// pub struct UnloadedImage {
//     pub offset: usize,
//     pub size: usize,
//     pub width: u32,
//     pub height: u32,
// }
// #[derive(Serialize, Deserialize)]
// pub struct UnloadedGeometry {

//     pub positions: Vec<[f32; 3]>,
//     pub normals: Vec<[f32; 3]>,
//     pub uvs: Vec<[f32; 2]>,
//     pub indices: Vec<u16>,
// }
// #[derive(Serialize, Deserialize)]
// pub struct UnloadedScene {
//     pub nodes: Vec<UnloadedSceneNode>,
//     pub images: Vec<UnloadedImage>,
//     pub geometries: Vec<UnloadedGeometry>,
//     #[serde(with = "serde_bytes")]
//     pub images_byte_buffer: Vec<u8>,
// }
