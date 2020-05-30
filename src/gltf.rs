use crate::*;
use glam::{Vec3, Mat4, Quat};
use serde::Deserialize;
use std::collections::HashMap;
use std::io::Read;

#[derive(Debug, Deserialize)]
struct Document {
    asset: Asset,
    nodes: Vec<Node>,
    buffers: Vec<Buffer>,
    bufferViews: Vec<BufferView>,
    accessors: Vec<Accesor>,
    samplers: Vec<Sampler>,
    images: Vec<Image>,
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
    mode: usize,
    attributes: HashMap<String, usize>,
}
#[derive(Debug, Deserialize)]
struct Scene {
    nodes: Vec<usize>,
}

pub fn load_gltf(file_name: &str) -> Option<Vec<render::Mesh>> {
    let document: Document = serde_json::from_str(&std::fs::read_to_string(file_name).ok()?).ok()?;

    let gltf_base_folder = file_name.rfind('/')
        .map(|idx| &file_name[0..idx+1])
        .unwrap_or("");

    let buffers: Vec<Vec<u8>> = document.buffers.iter().map(|b| {
        let mut result = Vec::new();
        std::fs::File::open(format!("{}{}", gltf_base_folder, b.uri)).ok()?.read_to_end(&mut result).ok()?;
        Some(result)
    }).collect::<Option<_>>()?;

    let mut meshes = Vec::new();
    
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
        let node = node_queue.pop()?;
        let parent_transform = transform_queue.pop().unwrap_or(Mat4::identity());

        let node_local_transform = {
            if let Some(m) = node.matrix {
                Mat4::from_cols_array(&m)
            } else {
                let t: Vec3 = node.translation.unwrap_or([0.0, 0.0, 0.0]).into(); 
                let r: Quat = node.rotation.unwrap_or([0.0, 0.0, 0.0, 1.0]).into();
                let s: Vec3 = node.translation.unwrap_or([1.0, 1.0, 1.0]).into();
                Mat4::from_translation(t) * Mat4::from_quat(r) * Mat4::from_scale(s)
            }
        };
        let transform = parent_transform * node_local_transform;

        for index in &node.children {
            node_queue.push(&document.nodes[*index]);
            transform_queue.push(transform);
        }

        let mesh = match node.mesh {
            Some(index) => &document.meshes[index],
            None => continue,
        };

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

            debug_assert!(positions.len() == normals.len() && positions.len() == uvs.len(),
                "there isn't the same amount of positions, normals and uvs.\npositions: {}, normals: {}, uvs: {}",
                positions.len(), normals.len(), uvs.len());

            // let mut vertices = Vec::new();
            for i in 0..positions.len() {
                let position = positions[i];
                let normal = normals[i];
                let uv = uvs[i];
                todo!(""); // vertices.push(render::Vertex{ position, normal, uv });
            }

            let indices: &[u16] = {
                let accessor = &document.accessors[primitive.indices];
                debug_assert!(accessor.componentType == 5123);
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
                let image = &document.images[document.textures[info.index].source];
                let image_file_name = format!("{}{}", gltf_base_folder, image.uri);
                todo!("texture_repository.load_from_file_name(image_file_name)")
            });
            let normal = material.normalTexture.as_ref().map(|info| {
                let image = &document.images[document.textures[info.index].source];
                let image_file_name = format!("{}{}", gltf_base_folder, image.uri);
                todo!("texture_repository.load_from_file_name(image_file_name)")
            });

            let base_diffuse_color = material.pbrMetallicRoughness.baseColorFactor;

            todo!();
            // let vertices = VertexBuffer::new(display, &vertices).unwrap();
            // let indices = IndexBuffer::new(display, PrimitiveType::TrianglesList, &indices).unwrap();
            // let transform = (y_up_to_z_up_transform * transform).to_cols_array_2d();
            // meshes.push(render::Mesh{ vertices, indices, transform, diffuse, normal, base_diffuse_color });
        }
    }

    Some(meshes)
}

