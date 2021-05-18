use std::borrow::Cow;

use image::{EncodableLayout, GenericImageView, ImageError, ImageFormat};

use crate::{
    catalog::Id,
    graphics::{Material, Mesh, Primitive, Texture},
    Catalog, Entity,
};

pub fn import(
    file_name: &str,
    entities: &mut Catalog<Entity>,
    meshes: &mut Catalog<Mesh>,
    materials: &mut Catalog<Material>,
    textures: &mut Catalog<Texture>,
) -> Vec<ImportGltfError> {
    let gltf = gltf::Gltf::open(file_name).unwrap();
    // let (document, buffers, images) = ::gltf::import(file_name).unwrap();
    let base_path = file_name[0..file_name.rfind("/").unwrap()].to_string();
    let mut importer = Importer {
        entities_catalog: entities,
        meshes_catalog: meshes,
        materials_catalog: materials,
        textures_catalog: textures,
        blob: gltf.blob,
        buffers: vec![],
        images: vec![],
        textures: vec![],
        materials: vec![],
        meshes: vec![],
        base_path,
    };

    let r = importer.import(gltf.document);
    if let Err(e) = &r {
        eprintln!("{}", e);
    }
    r.unwrap()
}
struct Importer<'catalogs> {
    blob: Option<Vec<u8>>,
    base_path: String,

    entities_catalog: &'catalogs mut Catalog<Entity>,
    meshes_catalog: &'catalogs mut Catalog<Mesh>,
    materials_catalog: &'catalogs mut Catalog<Material>,
    textures_catalog: &'catalogs mut Catalog<Texture>,

    // relate gltf indices to data or catalog ids
    buffers: Vec<Vec<u8>>,
    images: Vec<(Vec<u8>, usize, usize)>,
    textures: Vec<Id<Texture>>,
    materials: Vec<Id<Material>>,
    meshes: Vec<Id<Mesh>>,
}

impl<'catalogs> Importer<'catalogs> {
    fn import(
        &mut self,
        document: gltf::Document,
    ) -> Result<Vec<ImportGltfError>, ImportGltfError> {
        let mut errors = vec![];

        for buffer in document.buffers() {
            let b = self.import_gltf_buffer(buffer)?;
            self.buffers.push(b);
        }

        for image in document.images() {
            self.images.push(self.import_gltf_image(image)?);
        }

        for texture in document.textures() {
            let t = self.import_gltf_texture(texture)?;
            self.textures.push(t);
        }

        for material in document.materials() {
            let m = self.import_gltf_material(material)?;
            self.materials.push(m);
        }

        for mesh in document.meshes() {
            let m = self.import_gltf_mesh(mesh)?;
            self.meshes.push(m);
        }

        let scene = document.default_scene().unwrap();
        for node in scene.nodes() {
            if let Err(e) = self.import_gltf_node(node, None) {
                errors.push(e);
            }
        }

        Ok(errors)
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

    // result is (rgba bytes, width, height)
    fn import_gltf_image(
        &self,
        image: gltf::Image,
    ) -> Result<(Vec<u8>, usize, usize), ImportGltfError> {
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
                    None => parsed_mt
                };

                (Cow::from(data), mime_type)
            }
            gltf::image::Source::View { view, mime_type } => {
                let buffer = &self.buffers[view.buffer().index()];
                let data = &buffer[view.offset()..view.offset() + view.length()];
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

        let loaded = image::load_from_memory_with_format(&data, format)
            .map_err(|e| ImportGltfError::ImageLoadingFailed(image.index().to_string(), e))?;
        let (w, h) = (loaded.width() as usize, loaded.height() as usize);
        Ok((loaded.into_rgba().as_bytes().to_owned(), w, h))
    }

    fn import_gltf_texture(
        &mut self,
        texture: gltf::Texture,
    ) -> Result<Id<Texture>, ImportGltfError> {
        let (data, width, height) = &self.images[texture.source().index()];
        // TODO! samplers etc
        Ok(self
            .textures_catalog
            .add(Texture::from_rgba(&data, *width as i32, *height as i32)))
    }

    fn import_gltf_material(
        &mut self,
        material: gltf::Material,
    ) -> Result<Id<Material>, ImportGltfError> {
        let normal = match material.normal_texture().as_ref() {
            Some(info) => self.textures.get(info.texture().index()).copied(),
            None => None,
        };
        let diffuse = match material
            .pbr_metallic_roughness()
            .base_color_texture()
            .as_ref()
        {
            Some(info) => self.textures.get(info.texture().index()).copied(),
            None => None,
        };
        let base_diffuse_color = material.pbr_metallic_roughness().base_color_factor();
        Ok(self.materials_catalog.add(Material {
            normal,
            diffuse,
            base_diffuse_color,
        }))
    }

    fn import_gltf_mesh(&mut self, mesh: gltf::Mesh) -> Result<Id<Mesh>, ImportGltfError> {
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

            let material = match primitive.material().index() {
                Some(i) => self.materials[i],
                None => self.import_gltf_material(primitive.material())?, // i'm importing the default material, which doesn't make much sense
            };

            primitives.push(Primitive::new(
                &positions, &normals, &uvs, &indices, material,
            ));
        }
        Ok(self.meshes_catalog.add(Mesh { primitives }))
    }

    fn import_gltf_node(
        &mut self,
        node: gltf::Node,
        parent: Option<Id<Entity>>,
    ) -> Result<Id<Entity>, ImportGltfError> {
        let mesh = match node.mesh() {
            Some(mesh) => self.meshes.get(mesh.index()).copied(),
            None => None,
        };
        let entity = Entity {
            children: vec![],
            parent,
            mesh,
            transform: node.transform().matrix(),
        };
        let id = self.entities_catalog.add(entity);

        let mut children = vec![];
        for child in node.children() {
            let child_id = self.import_gltf_node(child, Some(id))?;
            children.push(child_id);
        }
        self.entities_catalog
            .get_mut(id)
            .ok_or(ImportGltfError::Unreachable)?
            .children = children;

        Ok(id)
    }
}

fn data_uri_to_bytes_and_type(uri: &str) -> Result<(Vec<u8>, &str), base64::DecodeError> {
    let bytes = base64::decode(&uri[uri.find(",").unwrap_or(0) + 1..])?;
    let mt = &uri[uri.find(":").unwrap()+1..uri.find(";").unwrap()];
    Ok((bytes, mt))
}

#[derive(thiserror::Error, Debug)]
pub enum ImportGltfError {
    #[error("io error: {0}")]
    IOError(#[from] std::io::Error),
    #[error("base 64 decode error: {0}")]
    Base64Error(#[from] base64::DecodeError),
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
    #[error("unreachable")]
    Unreachable,
}
