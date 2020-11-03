use crate::*;
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Document {
    #[serde(default)]
    extensions_used: Vec<String>,
    #[serde(default)]
    extensions_required: Vec<String>,
    #[serde(default)]
    pub accessors: Vec<Accesor>,
    #[serde(default)]
    pub animations: Vec<Animation>,
    pub asset: Asset,
    #[serde(default)]
    pub buffers: Vec<Buffer>,
    #[serde(default)]
    pub buffer_views: Vec<BufferView>,
    #[serde(default)]
    pub cameras: Vec<Camera>,
    #[serde(default)]
    pub images: Vec<Image>,
    #[serde(default)]
    pub materials: Vec<Material>,
    #[serde(default)]
    pub meshes: Vec<Mesh>,
    #[serde(default)]
    pub nodes: Vec<Node>,
    #[serde(default)]
    pub samplers: Vec<Sampler>,
    #[serde(default)]
    pub scene: Option<usize>,
    #[serde(default)]
    pub scenes: Vec<Scene>,
    #[serde(default)]
    pub skins: Vec<Skin>,
    #[serde(default)]
    pub textures: Vec<Texture>,
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Image {
    pub uri: Option<String>,
    pub mime_type: Option<String>,
    pub buffer_view: Option<usize>,
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
    pub byte_length: usize,
    pub uri: String,
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BufferView {
    pub buffer: usize,
    pub byte_length: usize,
    pub byte_offset: usize,
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Accesor {
    pub buffer_view: Option<usize>,
    #[serde(default)]
    pub byte_offset: usize,
    pub component_type: usize,
    #[serde(default)]
    pub normalized: bool,
    pub count: usize,
    #[serde(alias = "type")]
    pub _type: String,
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
    pub alpha_mode: Option<String>,
    pub double_sided: Option<bool>,
    pub pbr_metallic_roughness: PbrMetallicRoughness,
    pub normal_texture: Option<TextureInfo>,
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PbrMetallicRoughness {
    pub base_color_factor: [f32; 4],
    pub base_color_texture: Option<TextureInfo>,
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TextureInfo {
    pub index: usize,
    pub scale: Option<f32>,
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
    pub indices: usize,
    pub material: usize,
    pub mode: Option<usize>,
    pub attributes: HashMap<String, usize>,
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

impl GLTF {
    pub fn load_buffers(&self) -> Result<Vec<Vec<u8>>, AyudeError> {
        use std::io::Read;
        let buffers: Vec<Vec<u8>> = self.document.buffers.iter().map(|b| {
            let mut result = Vec::new();
            std::fs::File::open(format!("{}{}", self.gltf_base_folder, b.uri))?.read_to_end(&mut result)?;
            Ok(result)
        }).collect::<Result<_, AyudeError>>()?;
        Ok(buffers)
    }
    pub fn load_image(&self, uri: &str) -> Result<image::ImageBuffer<image::Rgba<u8>, Vec<u8>>, AyudeError> {
        let image_file_name = format!("{}{}", self.gltf_base_folder, uri);
        Ok(image::open(&image_file_name)?.into_rgba())
    }
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
