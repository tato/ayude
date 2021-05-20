use std::borrow::Cow;

use glam::Mat4;
use image::{DynamicImage, EncodableLayout, ImageError, ImageFormat};

use crate::{
    graphics::{
        texture::{MagFilter, MinFilter, TextureFormat, TextureWrap},
        Material, Mesh, Texture,
    },
    Entity, Transform,
};

// notes:
// for me, a gltf will only contain 1 entity, with 1 mesh, with 1 skin, with a set of
// animations, textures and materials. the entity could be and in fact will probably
// be a tree of nodes, but from outside it will seem a single object.
// if possible, the set of nodes, meshes and skins in the gltf file should be
// condensed into 1. if not, we'd have to choose 1 and discard the rest.
//
// in reality, a gltf should not import as an "Entity", it should maybe act as an
// entity blueprint, or even forget the concept of entity and just import the mesh.
//
// another option would be to borrow the concept of a "scene" from the gltf. in that
// case, the gltf would import into a list of "scenes", and the meshes, textures,
// materials and so on could be shared between these "scenes". then, i'd have a
// "SceneEntity" object that i could instantiate and would refer to one of these
// "scenes" as a blueprint.
//
// one possible problem with this last approach could be the cleanup stage. if we
// have so many meshes and textures and so on lying around with no reference to
// their parent scene, we could end up with some storage issues when we no longer
// need the scene. a mesh could be used by multiple scenes, so that stuff would
// need to be checked before it's cleaned up. the solution to that is each
// "scene" or entity blueprint or whatever having its own copy of a mesh, texture,
// etc that it needs. OR these meshes, textures, etc being attached to certain
// "stages" or "levels" of the game which know they need a set of scenes or
// whatever. this might not be too hard, it could be inferred from the construction
// of the levels themselves.
//
// the clearest of the fuzzy ideas floating around in my mind right now is:
//   [gltf files] --level editor--> level file --engine loader--> [entity blueprints]
// i don't have a semblance of a level editor right now, so the association between
// multiple gltf files could be inferred from filesystem or some other simple system
// for now. in the future if these are imported to the engine and stuff they will
// have the possibility of deduping textures and so on, but for now the conclusion is
// IMPORT SCENES FROM EACH GLTF FILE AND DON'T HAVE A GLOBAL MESH, TEXTURE, ETC THING

pub fn import(file_name: &str) -> Result<Entity, ImportGltfError> {
    let gltf = gltf::Gltf::open(file_name)?;
    let base_path = file_name[0..file_name.rfind("/").unwrap()].to_string();
    let mut importer = Importer {
        blob: gltf.blob,
        buffers: vec![],
        images: vec![],
        node_parents: vec![None; gltf.document.nodes().count()],
        textures: vec![None; gltf.document.textures().count()],
        materials: vec![None; gltf.document.materials().count()],
        meshes: vec![None; gltf.document.meshes().count()],
        base_path,
    };

    importer.import(gltf.document)
}
struct Importer {
    base_path: String,
    blob: Option<Vec<u8>>,

    buffers: Vec<Vec<u8>>,
    images: Vec<(Vec<u8>, u32, u32, TextureFormat)>,

    node_parents: Vec<Option<usize>>,

    textures: Vec<Option<Texture>>,
    materials: Vec<Option<Material>>,
    meshes: Vec<Option<Vec<Mesh>>>,
}

impl Importer {
    fn import(&mut self, document: gltf::Document) -> Result<Entity, ImportGltfError> {
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

        // store the parent of each node so we can traverse the tree backwards
        for node in scene.nodes() {
            for child in node.children() {
                self.node_parents[child.index()] = Some(node.index());
            }
        }

        let (meshes, mesh_transforms) = self.collect_scene_meshes(&document, scene)?;

        let skin = None;
        let transform = Transform::new([
            [1.0, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ]);

        Ok(Entity {
            meshes,
            mesh_transforms,
            skin,
            transform,
        })
    }

    fn collect_scene_meshes(
        &mut self,
        document: &gltf::Document,
        scene: gltf::Scene,
    ) -> Result<(Vec<Mesh>, Vec<Transform>), ImportGltfError> {
        let mut meshes = vec![];
        let mut transforms = vec![];

        for node in scene.nodes() {
            if let Some(mesh) = node.mesh() {
                // calculate transform by multiplying all parents or whatever
                let transform = {
                    let mut result = Mat4::identity();
                    let mut current = node;
                    loop {
                        result = result * Mat4::from_cols_array_2d(&current.transform().matrix());
                        if let Some(parent) = self.node_parents[current.index()] {
                            current = match document.nodes().nth(parent) {
                                Some(n) => n,
                                None => break,
                            };
                        } else {
                            break;
                        }
                    }
                    result
                };

                let import = self.import_gltf_mesh(mesh)?;                
                
                let transform = Transform::new(transform.to_cols_array_2d());
                for _ in 0..import.len() {
                    transforms.push(transform.clone());
                }

                meshes.extend(import);
            }
        }

        Ok((meshes, transforms))
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
    ) -> Result<(Vec<u8>, u32, u32, TextureFormat), ImportGltfError> {
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
            DynamicImage::ImageRgb8(rgb) => Ok((
                rgb.as_bytes().to_owned(),
                rgb.width(),
                rgb.height(),
                TextureFormat::RGB,
            )),
            DynamicImage::ImageRgba8(rgba) => Ok((
                rgba.as_bytes().to_owned(),
                rgba.width(),
                rgba.height(),
                TextureFormat::RGBA,
            )),
            _ => {
                let rgba = image.into_rgba();
                Ok((
                    rgba.as_bytes().to_owned(),
                    rgba.width(),
                    rgba.height(),
                    TextureFormat::RGBA,
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

        let mut builder = Texture::builder(&data, *width as u16, *height as u16, *format)
            .wrap_s(match sampler.wrap_s() {
                gltf::texture::WrappingMode::ClampToEdge => TextureWrap::ClampToEdge,
                gltf::texture::WrappingMode::MirroredRepeat => TextureWrap::MirroredRepeat,
                gltf::texture::WrappingMode::Repeat => TextureWrap::Repeat,
            })
            .wrap_t(match sampler.wrap_t() {
                gltf::texture::WrappingMode::ClampToEdge => TextureWrap::ClampToEdge,
                gltf::texture::WrappingMode::MirroredRepeat => TextureWrap::MirroredRepeat,
                gltf::texture::WrappingMode::Repeat => TextureWrap::Repeat,
            });

        if let Some(min_filter) = sampler.min_filter() {
            builder = builder.min_filter(match min_filter {
                gltf::texture::MinFilter::Nearest => MinFilter::Nearest,
                gltf::texture::MinFilter::Linear => MinFilter::Linear,
                gltf::texture::MinFilter::NearestMipmapNearest => MinFilter::NearestMipmapNearest,
                gltf::texture::MinFilter::LinearMipmapNearest => MinFilter::LinearMipmapNearest,
                gltf::texture::MinFilter::NearestMipmapLinear => MinFilter::NearestMipmapLinear,
                gltf::texture::MinFilter::LinearMipmapLinear => MinFilter::LinearMipmapNearest,
            });
        }

        if let Some(mag_filter) = sampler.mag_filter() {
            builder = builder.mag_filter(match mag_filter {
                gltf::texture::MagFilter::Nearest => MagFilter::Nearest,
                gltf::texture::MagFilter::Linear => MagFilter::Linear,
            });
        }

        let texture = builder.build();
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
        })
    }

    fn import_gltf_mesh(&mut self, mesh: gltf::Mesh) -> Result<Vec<Mesh>, ImportGltfError> {
        let mesh_index = mesh.index();
        if let Some(m) = self
            .meshes
            .get(mesh_index)
            .ok_or(ImportGltfError::UnknownMeshIndex(mesh_index))?
        {
            return Ok(m.clone());
        }

        let mut primitives = vec![];
        for primitive in mesh.primitives() {
            let reader =
                primitive.reader(|buffer| self.buffers.get(buffer.index()).map(Vec::as_slice));

            let positions = reader
                .read_positions()
                .ok_or(ImportGltfError::RequiredMeshPropertyMissing(
                    "positions",
                    mesh.index(),
                    primitive.index(),
                ))?
                .collect::<Vec<_>>();

            let normals = reader
                .read_normals()
                .ok_or(ImportGltfError::RequiredMeshPropertyMissing(
                    "normals",
                    mesh.index(),
                    primitive.index(),
                ))?
                .collect::<Vec<_>>();

            let uvs = reader
                .read_tex_coords(0)
                .ok_or(ImportGltfError::RequiredMeshPropertyMissing(
                    "uvs",
                    mesh.index(),
                    primitive.index(),
                ))?
                .into_f32()
                .collect::<Vec<_>>();

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

            primitives.push(Mesh::new(&positions, &normals, &uvs, &indices, &material));
        }

        Ok(primitives)
    }

    // fn import_gltf_skin(&mut self, skin: gltf::Skin) -> Result<Skin, ImportGltfError> {
    //     let skin_index = skin.index();
    //     if let Some(sk) = self.skins.get(skin_index).ok_or(ImportGltfError::UnknownSkinIndex(skin_index))? {
    //         return Ok(sk.clone());
    //     }

    //     let mut joints = vec![];
    //     for joint in skin.joints() {
    //         let joint_index = join.index();
    //         joints.push(
    //             self.nodes
    //                 .get(joint_index)
    //                 .copied()
    //                 .ok_or(ImportGltfError::UnknownNodeIndex(joint_index))?,
    //         );
    //     }

    //     Ok(Skin { joints })
    // }

    // fn import_gltf_partial_node(
    //     &mut self,
    //     node: gltf::Node,
    // ) -> Result<Id<Entity>, ImportGltfError> {
    //     let mesh = match node.mesh() {
    //         Some(mesh) => self.meshes.get(mesh.index()).copied(),
    //         None => None,
    //     };
    //     let entity = Entity {
    //         children: vec![],
    //         parent: None,
    //         skin: None,
    //         mesh,
    //         transform: node.transform().matrix(),
    //     };
    //     Ok(self.entities_catalog.add(entity))
    // }

    // // todo! prevent possible panics
    // fn complete_gltf_node_import(&mut self, node: gltf::Node) {
    //     for child in node.children() {
    //         self.entities_catalog
    //             .get_mut(self.nodes[node.index()])
    //             .unwrap()
    //             .children
    //             .push(self.nodes[child.index()]);
    //         self.entities_catalog
    //             .get_mut(self.nodes[child.index()])
    //             .unwrap()
    //             .parent = Some(self.nodes[node.index()]);
    //     }
    //     if let Some(skin) = node.skin() {
    //         self.entities_catalog
    //             .get_mut(self.nodes[node.index()])
    //             .unwrap()
    //             .skin = Some(self.skins[skin.index()]);
    //     }
    // }
}

fn data_uri_to_bytes_and_type(uri: &str) -> Result<(Vec<u8>, &str), base64::DecodeError> {
    let bytes = base64::decode(&uri[uri.find(",").unwrap_or(0) + 1..])?;
    let mt = &uri[uri.find(":").unwrap() + 1..uri.find(";").unwrap()];
    Ok((bytes, mt))
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
    #[error("unreachable")]
    Unreachable,
}
