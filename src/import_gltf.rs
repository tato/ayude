use std::{borrow::Cow, convert::TryInto, iter::repeat};

use glam::Mat4;
use image::{DynamicImage, EncodableLayout, ImageError, ImageFormat};
use smallvec::SmallVec;

use crate::{
    graphics::{
        GraphicsContext, Material, Mesh, Texture, TextureDescription, UniformBuffer, Vertex,
    },
    transform::Transform,
    Node, Scene, Skin,
};

pub fn import_default_scene(
    file_name: &str,
    graphics: &GraphicsContext,
) -> Result<Scene, ImportGltfError> {
    let gltf = gltf::Gltf::open(file_name)?;
    let base_path = file_name[0..file_name.rfind("/").unwrap()].to_string();
    let mut importer = Importer {
        blob: gltf.blob,
        buffers: vec![],
        images: vec![],
        textures: vec![None; gltf.document.textures().count()],
        materials: vec![None; gltf.document.materials().count()],
        meshes: vec![None; gltf.document.meshes().count()],
        base_path,
        graphics,
    };

    importer.import_default_scene(gltf.document)
}
struct Importer<'gfx> {
    base_path: String,
    blob: Option<Vec<u8>>,

    buffers: Vec<Vec<u8>>,
    images: Vec<(Vec<u8>, u32, u32, wgpu::TextureFormat)>,

    textures: Vec<Option<Texture>>,
    materials: Vec<Option<Material>>,
    meshes: Vec<Option<Vec<(Mesh, Material)>>>,

    graphics: &'gfx GraphicsContext,
}

impl<'gfx> Importer<'gfx> {
    fn import_default_scene(&mut self, document: gltf::Document) -> Result<Scene, ImportGltfError> {
        // check if document has default scene
        let scene = document
            .default_scene()
            .expect("gltf document should have default scene");

        // pre-import buffers and images
        for buffer in document.buffers() {
            let b = self.import_gltf_buffer(buffer)?;
            self.buffers.push(b);
        }

        for image in document.images() {
            self.images.push(self.import_gltf_image(image)?);
        }

        let mut nodes = vec![];

        let root_nodes = scene
            .nodes()
            .map(|it| map_node_to_u16_index(&it))
            .collect::<Result<SmallVec<[u16; 4]>, ImportGltfError>>()?;

        let mut node_stack: Vec<(gltf::Node, Option<u16>)> =
            scene.nodes().zip(repeat(None)).collect();

        loop {
            let (node, parent) = match node_stack.pop() {
                Some(it) => it,
                None => break,
            };

            let node_index = map_node_to_u16_index(&node)?;

            node_stack.extend(node.children().zip(repeat(Some(node_index))));

            let children = node
                .children()
                .map(|it| map_node_to_u16_index(&it))
                .collect::<Result<SmallVec<[u16; 4]>, ImportGltfError>>()?;

            let transform = Transform::from(Mat4::from_cols_array_2d(&node.transform().matrix()));

            let meshes = match node.mesh() {
                Some(mesh) => self.import_gltf_mesh(mesh)?,
                None => vec![],
            };

            let skin = match node.skin() {
                Some(skin) => {
                    let joints = skin
                        .joints()
                        .map(|it| map_node_to_u16_index(&it))
                        .collect::<Result<SmallVec<[u16; 4]>, ImportGltfError>>()?;

                    let skeleton = match skin.skeleton().map(|it| map_node_to_u16_index(&it)) {
                        Some(Ok(it)) => Ok(Some(it)),
                        Some(Err(e)) => Err(e),
                        None => Ok(None),
                    }?;

                    let inverse_bind_matrices = skin
                        .reader(|buffer| self.buffers.get(buffer.index()).map(Vec::as_slice))
                        .read_inverse_bind_matrices()
                        .map(|it| {
                            it.map(|mat| Transform::from(Mat4::from_cols_array_2d(&mat)))
                                .collect()
                        })
                        .unwrap_or_else(|| vec![Transform::from(Mat4::IDENTITY); joints.len()]);

                    Some(Skin {
                        joints,
                        skeleton,
                        inverse_bind_matrices,
                    })
                }
                None => None,
            };

            nodes.push((
                node_index,
                Node {
                    parent,
                    children,
                    transform,
                    meshes,
                    skin,
                    name: node.name().map(str::to_string),
                },
            ));
        }

        nodes.sort_unstable_by(|a, b| a.0.cmp(&b.0));

        let nodes = nodes.into_iter().map(|it| it.1).collect();

        let transform = Transform::from(Mat4::IDENTITY);

        Ok(Scene {
            transform,
            nodes,
            root_nodes,
        })
    }

    fn import_gltf_buffer(&mut self, buffer: gltf::Buffer) -> Result<Vec<u8>, ImportGltfError> {
        match buffer.source() {
            gltf::buffer::Source::Bin => {
                self.blob.take().ok_or(ImportGltfError::BinSectionNotFound)
            }
            gltf::buffer::Source::Uri(uri) => {
                if uri.starts_with("data:") {
                    Ok(data_uri_to_bytes_and_type(uri)?.0)
                } else {
                    Ok(std::fs::read(format!("{}/{}", self.base_path, uri))?)
                }
            }
        }
    }

    // result is (rgba bytes, width, height, format)
    fn import_gltf_image(
        &self,
        image: gltf::Image,
    ) -> Result<(Vec<u8>, u32, u32, wgpu::TextureFormat), ImportGltfError> {
        let (data, mime_type) = match image.source() {
            gltf::image::Source::Uri { uri, mime_type } => {
                let (data, parsed_mt) = if uri.starts_with("data:") {
                    data_uri_to_bytes_and_type(uri)?
                } else {
                    let bytes = std::fs::read(&format!("{}/{}", self.base_path, uri))?;
                    let format = if uri.ends_with(".png") {
                        "image/png"
                    } else if uri.ends_with(".jpg") || uri.ends_with(".jpeg") {
                        "image/jpeg"
                    } else {
                        "application/octet-stream"
                    };
                    (bytes, format)
                };

                let mime_type = match mime_type {
                    Some(mt) => mt,
                    None => parsed_mt,
                };

                (Cow::from(data), mime_type)
            }
            gltf::image::Source::View { view, mime_type } => {
                let buffer_index = view.buffer().index();
                let buffer = &self
                    .buffers
                    .get(buffer_index)
                    .ok_or(ImportGltfError::UnknownBufferIndex(buffer_index))?;
                let from = view.offset();
                let to = view.offset() + view.length();
                let data = buffer
                    .get(from..to)
                    .ok_or(ImportGltfError::BufferRangeOutOfBounds(
                        buffer_index,
                        from,
                        to,
                    ))?;
                (Cow::from(data), mime_type)
            }
        };

        let format = match mime_type {
            "image/jpeg" => Ok(ImageFormat::Jpeg),
            "image/png" => Ok(ImageFormat::Png),
            fmt => Err(ImportGltfError::UnknownImageFormat(
                fmt.to_string(),
                image.index(),
            )),
        }?;

        let image = image::load_from_memory_with_format(&data, format)
            .map_err(|e| ImportGltfError::ImageLoadingFailed(image.index().to_string(), e))?;
        match image {
            DynamicImage::ImageRgba8(rgba) => Ok((
                rgba.as_bytes().to_owned(),
                rgba.width(),
                rgba.height(),
                wgpu::TextureFormat::Rgba8Unorm,
            )),
            _ => {
                let rgba = image.into_rgba8();
                Ok((
                    rgba.as_bytes().to_owned(),
                    rgba.width(),
                    rgba.height(),
                    wgpu::TextureFormat::Rgba8Unorm,
                ))
            }
        }
    }

    fn import_gltf_texture(&mut self, texture: gltf::Texture) -> Result<Texture, ImportGltfError> {
        let texture_index = texture.index();
        if let Some(tex) = self
            .textures
            .get(texture_index)
            .ok_or(ImportGltfError::UnknownTextureIndex(texture_index))?
        {
            return Ok(tex.clone());
        }

        let image_index = texture.source().index();
        let (data, width, height, format) = &self
            .images
            .get(image_index)
            .ok_or(ImportGltfError::UnknownImageIndex(image_index))?;

        let sampler = texture.sampler();

        let mut desc = TextureDescription::new(data, *width, *height, *format)
            .wrap_s(match sampler.wrap_s() {
                gltf::texture::WrappingMode::ClampToEdge => wgpu::AddressMode::ClampToEdge,
                gltf::texture::WrappingMode::MirroredRepeat => wgpu::AddressMode::MirrorRepeat,
                gltf::texture::WrappingMode::Repeat => wgpu::AddressMode::Repeat,
            })
            .wrap_t(match sampler.wrap_s() {
                gltf::texture::WrappingMode::ClampToEdge => wgpu::AddressMode::ClampToEdge,
                gltf::texture::WrappingMode::MirroredRepeat => wgpu::AddressMode::MirrorRepeat,
                gltf::texture::WrappingMode::Repeat => wgpu::AddressMode::Repeat,
            });

        if let Some(min_filter) = sampler.min_filter() {
            desc = desc.min_filter(match min_filter {
                gltf::texture::MinFilter::Nearest => wgpu::FilterMode::Nearest,
                gltf::texture::MinFilter::Linear => wgpu::FilterMode::Linear,
                _ => unimplemented!(),
            });
        }

        if let Some(mag_filter) = sampler.mag_filter() {
            desc = desc.mag_filter(match mag_filter {
                gltf::texture::MagFilter::Nearest => wgpu::FilterMode::Nearest,
                gltf::texture::MagFilter::Linear => wgpu::FilterMode::Linear,
            });
        }

        let texture = self.graphics.create_texture(&desc);
        Ok(texture)
    }

    fn import_gltf_material(
        &mut self,
        material: gltf::Material,
    ) -> Result<Material, ImportGltfError> {
        if let Some(index) = material.index() {
            if let Some(mat) = self
                .materials
                .get(index)
                .ok_or(ImportGltfError::UnknownMaterialIndex(index))?
            {
                return Ok(mat.clone());
            }
        }

        let normal = match material.normal_texture().as_ref() {
            Some(info) => Some(self.import_gltf_texture(info.texture())?),
            None => None,
        };
        let diffuse = match material
            .pbr_metallic_roughness()
            .base_color_texture()
            .as_ref()
        {
            Some(info) => Some(self.import_gltf_texture(info.texture())?),
            None => None,
        };
        let base_diffuse_color = material.pbr_metallic_roughness().base_color_factor();
        Ok(Material {
            normal,
            diffuse,
            base_diffuse_color,
            shaded: true,
        })
    }

    fn import_gltf_mesh(
        &mut self,
        mesh: gltf::Mesh,
    ) -> Result<Vec<(Mesh, UniformBuffer, Material)>, ImportGltfError> {
        let mesh_index = mesh.index();
        if let Some(m) = self
            .meshes
            .get(mesh_index)
            .ok_or(ImportGltfError::UnknownMeshIndex(mesh_index))?
        {
            return Ok(m
                .iter()
                .map(|(mesh, mat)| {
                    (
                        mesh.clone(),
                        self.graphics.create_uniform_buffer(),
                        mat.clone(),
                    )
                })
                .collect());
        }

        let mut primitives = vec![];
        for primitive in mesh.primitives() {
            let reader =
                primitive.reader(|buffer| self.buffers.get(buffer.index()).map(Vec::as_slice));

            let mut positions =
                reader
                    .read_positions()
                    .ok_or(ImportGltfError::RequiredMeshPropertyMissing(
                        "positions",
                        mesh.index(),
                        primitive.index(),
                    ))?;

            let mut normals =
                reader
                    .read_normals()
                    .ok_or(ImportGltfError::RequiredMeshPropertyMissing(
                        "normals",
                        mesh.index(),
                        primitive.index(),
                    ))?;

            let mut tex_coords = reader
                .read_tex_coords(0)
                .ok_or(ImportGltfError::RequiredMeshPropertyMissing(
                    "uvs",
                    mesh.index(),
                    primitive.index(),
                ))?
                .into_f32();

            let mut vertices: Vec<Vertex> = Vec::with_capacity(positions.len());
            for _ in 0..positions.len() {
                let p = positions.next().unwrap();
                let position = [p[0], p[1], p[2], 1.0];
                let normal = normals.next().unwrap();
                let tex_coord = {
                    let val = tex_coords.next().unwrap();
                    [val[0], val[1]]
                };
                let vertex = Vertex {
                    position,
                    normal,
                    tex_coord,
                };
                vertices.push(vertex);
            }

            let indices = reader
                .read_indices()
                .ok_or(ImportGltfError::RequiredMeshPropertyMissing(
                    "indices",
                    mesh.index(),
                    primitive.index(),
                ))?
                .into_u32()
                .map(|it| it as u16) // TODO! this sucks
                .collect::<Vec<_>>();

            let material = self.import_gltf_material(primitive.material())?;

            let mesh = self.graphics.create_mesh(&vertices, &indices);
            let ub = self.graphics.create_uniform_buffer();

            primitives.push((mesh, ub, material.clone()));
        }

        Ok(primitives)
    }
}

fn data_uri_to_bytes_and_type(uri: &str) -> Result<(Vec<u8>, &str), base64::DecodeError> {
    let bytes = base64::decode(&uri[uri.find(",").unwrap_or(0) + 1..])?;
    let mt = &uri[uri.find(":").unwrap() + 1..uri.find(";").unwrap()];
    Ok((bytes, mt))
}

fn map_node_to_u16_index(node: &gltf::Node) -> Result<u16, ImportGltfError> {
    node.index()
        .try_into()
        .map_err(|_| ImportGltfError::NodeIndexOutOfRange(node.index()))
}

#[derive(thiserror::Error, Debug)]
pub enum ImportGltfError {
    #[error("io error: {0}")]
    IOError(#[from] std::io::Error),
    #[error("base 64 decode error: {0}")]
    Base64Error(#[from] base64::DecodeError),
    #[error("error while loading source gltf: {0}")]
    GltfError(#[from] gltf::Error),
    #[error("image loading failed for file '{0}': {1}")]
    ImageLoadingFailed(String, ImageError),
    #[error("unknown image format '{0:?}' for image {1}")]
    UnknownImageFormat(String, usize),
    #[error("binary section of gltf not found")]
    BinSectionNotFound,
    #[error(
        "required property '{0}' is missing for mesh with index {1} and primitive with index {2}"
    )]
    RequiredMeshPropertyMissing(&'static str, usize, usize),
    #[error("unknown buffer index {0}")]
    UnknownBufferIndex(usize),
    #[error("buffer {0} has a view with range ({1}..{2}) that is out of bounds")]
    BufferRangeOutOfBounds(usize, usize, usize),
    #[error("unknown image index {0}")]
    UnknownImageIndex(usize),
    #[error("unknown material index {0}")]
    UnknownMaterialIndex(usize),
    #[error("unknown node index {0}")]
    UnknownNodeIndex(usize),
    #[error("unknown mesh index {0}")]
    UnknownMeshIndex(usize),
    #[error("unknown texture index {0}")]
    UnknownTextureIndex(usize),
    #[error("unkown skin index {0}")]
    UnknownSkinIndex(usize),
    #[error("node index {0} out of range")]
    NodeIndexOutOfRange(usize),
    #[error("unreachable")]
    Unreachable,
}
