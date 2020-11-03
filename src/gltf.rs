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