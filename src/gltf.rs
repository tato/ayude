use crate::*;
use glam::{Vec3, Mat4, Quat};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::io::Read;

// todo! accurate deserialize struct. maybe copy from gltf lib.
#[derive(Debug, Deserialize)]
struct Document {
    asset: Asset,
    nodes: Vec<Node>,
    buffers: Vec<Buffer>,
    bufferViews: Vec<BufferView>,
    accessors: Vec<Accesor>,
    #[serde(default)]
    samplers: Vec<Sampler>,
    #[serde(default)]
    images: Vec<Image>,
    #[serde(default)]
    textures: Vec<Texture>,
    materials: Vec<Material>,
    meshes: Vec<Mesh>,
    scene: Option<usize>,
    scenes: Vec<Scene>,
}
#[derive(Debug, Deserialize)]
struct Asset {
    generator: String,
    version: String,
}
#[derive(Debug, Deserialize)]
struct Node {
    #[serde(default)]
    children: Vec<usize>,
    mesh: Option<usize>,
    matrix: Option<[f32; 16]>,
    translation: Option<[f32; 3]>,
    rotation: Option<[f32; 4]>,
    scale: Option<[f32; 3]>,
}
#[derive(Debug, Deserialize)]
struct Buffer {
    byteLength: usize,
    uri: String,
}
#[derive(Debug, Deserialize)]
struct BufferView {
    buffer: usize,
    byteLength: usize,
    byteOffset: usize,
}
#[derive(Debug, Deserialize)]
struct Accesor {
    bufferView: usize,
    componentType: usize,
    #[serde(alias = "type")]
    _type: String,
    count: usize,
    #[serde(default)]
    byteOffset: usize,
}
#[derive(Debug, Deserialize)]
struct Sampler {
}
#[derive(Debug, Deserialize)]
struct Image {
    uri: String,
    mimeType: Option<String>,
}
#[derive(Debug, Deserialize)]
struct Texture {
    sampler: usize,
    source: usize,
}
#[derive(Debug, Deserialize)]
struct Material {
    alphaMode: Option<String>,
    doubleSided: Option<bool>,
    pbrMetallicRoughness: PbrMetallicRoughness,
    normalTexture: Option<TextureInfo>,
}
#[derive(Debug, Deserialize)]
struct PbrMetallicRoughness {
    baseColorFactor: [f32; 4],
    baseColorTexture: Option<TextureInfo>,
}
#[derive(Debug, Deserialize)]
struct TextureInfo {
    index: usize,
    scale: Option<f32>,
}
#[derive(Debug, Deserialize)]
struct Mesh {
    primitives: Vec<Primitive>
}
#[derive(Debug, Deserialize)]
struct Primitive {
    indices: usize,
    material: usize,
    mode: Option<usize>,
    attributes: HashMap<String, usize>,
}
#[derive(Debug, Deserialize)]
struct Scene {
    nodes: Vec<usize>,
}

pub fn load(file_name: &str) -> Result<UnloadedScene, AyudeError> {
    let document: Document = serde_json::from_str(&std::fs::read_to_string(file_name)?)?;

    let gltf_base_folder = file_name.rfind('/')
        .map(|idx| &file_name[0..idx+1])
        .unwrap_or("");

    let buffers: Vec<Vec<u8>> = document.buffers.iter().map(|b| {
        let mut result = Vec::new();
        std::fs::File::open(format!("{}{}", gltf_base_folder, b.uri))?.read_to_end(&mut result)?;
        Ok(result)
    }).collect::<Result<_, AyudeError>>()?;

    let mut images = Vec::new();
    let mut images_byte_buffer = Vec::new();
    for image in &document.images {
        let image_file_name = format!("{}{}", gltf_base_folder, image.uri);
        let loaded = image::open(&image_file_name)?.into_rgba();
        let width = loaded.width();
        let height = loaded.height();
        let bytes = image::DynamicImage::ImageRgba8(loaded).to_bytes();
        images.push(UnloadedImage{ offset: images_byte_buffer.len(), size: bytes.len(), width, height });
        images_byte_buffer.extend(bytes);
    }

    let mut nodes = Vec::new();
    let mut geometries = Vec::new();
    
    let y_up_to_z_up_transform = Mat4::from_cols_array(&[
        0.0, 1.0, 0.0, 0.0,
        0.0, 0.0, 1.0, 0.0,
        1.0, 0.0, 0.0, 0.0,
        0.0, 0.0, 0.0, 1.0f32,
    ]);
    
    let mut node_queue = Vec::new();
    let mut transform_queue = Vec::new();

    let default_scene_nodes = document.scenes[document.scene.unwrap_or(0)].nodes.iter()
        .map(|index| &document.nodes[*index]);
    node_queue.extend(default_scene_nodes);

    while !node_queue.is_empty() {
        let node = node_queue.pop().unwrap();
        let parent_transform = transform_queue.pop().unwrap_or(Mat4::identity());

        let mesh = match node.mesh {
            Some(index) => &document.meshes[index],
            None => continue,
        };

        let node_local_transform = {
            if let Some(m) = node.matrix {
                Mat4::from_cols_array(&m)
            } else {
                let t: Vec3 = node.translation.unwrap_or([0.0, 0.0, 0.0]).into(); 
                let r: Quat = node.rotation.unwrap_or([0.0, 0.0, 0.0, 1.0]).into();
                let s: Vec3 = node.scale.unwrap_or([1.0, 1.0, 1.0]).into();
                Mat4::from_translation(t) * Mat4::from_quat(r) * Mat4::from_scale(s)
            }
        };
        let transform = parent_transform * node_local_transform;

        for index in &node.children {
            node_queue.push(&document.nodes[*index]);
            transform_queue.push(transform);
        }

        for primitive in &mesh.primitives {
            let positions: &[[f32; 3]] = {
                let accessor = &document.accessors[primitive.attributes["POSITION"]];
                debug_assert!(accessor.componentType == 5126);
                debug_assert!(accessor._type == "VEC3");
                let view = &document.bufferViews[accessor.bufferView];
                let buffer = &buffers[view.buffer][view.byteOffset..(view.byteOffset+view.byteLength)];
                unsafe {
                    let ptr = std::mem::transmute(buffer.as_ptr());
                    std::slice::from_raw_parts(ptr, buffer.len() / 12)
                }
            };
            let normals: &[[f32; 3]] = {
                let accessor = &document.accessors[primitive.attributes["NORMAL"]];
                debug_assert!(accessor.componentType == 5126);
                debug_assert!(accessor._type == "VEC3");
                let view = &document.bufferViews[accessor.bufferView];
                let buffer = &buffers[view.buffer][view.byteOffset..(view.byteOffset+view.byteLength)];
                unsafe {
                    let ptr = std::mem::transmute(buffer.as_ptr());
                    std::slice::from_raw_parts(ptr, buffer.len() / 12)
                }
            };
            let uvs: &[[f32; 2]] = {
                let accessor = &document.accessors[primitive.attributes["TEXCOORD_0"]];
                debug_assert!(accessor.componentType == 5126);
                debug_assert!(accessor._type == "VEC2");
                let view = &document.bufferViews[accessor.bufferView];
                let buffer = &buffers[view.buffer][view.byteOffset..(view.byteOffset+view.byteLength)];
                unsafe {
                    let ptr = std::mem::transmute(buffer.as_ptr());
                    std::slice::from_raw_parts(ptr, buffer.len() / 8)
                }
            };

            let indices: &[u16] = {
                let accessor = &document.accessors[primitive.indices];
                debug_assert!(accessor.componentType == 5123, "accessor.componentType ({}) == 5123", accessor.componentType);
                debug_assert!(accessor._type == "SCALAR");
                let view = &document.bufferViews[accessor.bufferView];
                let buffer = &buffers[view.buffer][view.byteOffset..(view.byteOffset+view.byteLength)];
                unsafe {
                    let ptr = std::mem::transmute(buffer.as_ptr());
                    std::slice::from_raw_parts(ptr, buffer.len() / 2)
                }
            };

            let material = &document.materials[primitive.material];
            let diffuse = material.pbrMetallicRoughness.baseColorTexture.as_ref().map(|info| {
                document.textures[info.index].source
            });
            let normal = material.normalTexture.as_ref().map(|info| {
                document.textures[info.index].source
            });

            let base_diffuse_color = material.pbrMetallicRoughness.baseColorFactor;

            let positions = positions.to_vec();
            let normals = normals.to_vec();
            let uvs = uvs.to_vec();
            let indices = indices.to_vec();
            let transform = (y_up_to_z_up_transform * transform).to_cols_array_2d();

            let geometry = UnloadedGeometry{positions, normals, uvs, indices};
            geometries.push(geometry);

            let geometry_id = geometries.len() as u32 - 1;

            nodes.push(UnloadedSceneNode{ geometry_id, transform, diffuse, normal, base_diffuse_color });
        }
    }

    Ok(UnloadedScene{ nodes, images, geometries, images_byte_buffer })
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UnloadedSceneNode {
    pub geometry_id: u32,

    pub transform: [[f32; 4]; 4],

    pub diffuse: Option<usize>,
    pub normal: Option<usize>,

    pub base_diffuse_color: [f32; 4],
}
#[derive(Serialize, Deserialize)]
pub struct UnloadedImage {
    pub offset: usize,
    pub size: usize,
    pub width: u32,
    pub height: u32,
}
#[derive(Serialize, Deserialize)]
pub struct UnloadedGeometry {

    pub positions: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
    pub uvs: Vec<[f32; 2]>,
    pub indices: Vec<u16>,
}
#[derive(Serialize, Deserialize)]
pub struct UnloadedScene {
    pub nodes: Vec<UnloadedSceneNode>,
    pub images: Vec<UnloadedImage>,
    pub geometries: Vec<UnloadedGeometry>,
    #[serde(with = "serde_bytes")]
    pub images_byte_buffer: Vec<u8>,
}