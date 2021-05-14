use image::EncodableLayout;

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
    let (document, buffers, images) = ::gltf::import(file_name).unwrap();
    let mut importer = Importer {
        entities,
        meshes,
        materials,
        textures,
        buffers,
        images,
    };
    importer.import(document)
}
struct Importer<'catalogs> {
    entities: &'catalogs mut Catalog<Entity>,
    meshes: &'catalogs mut Catalog<Mesh>,
    materials: &'catalogs mut Catalog<Material>,
    textures: &'catalogs mut Catalog<Texture>,
    buffers: Vec<::gltf::buffer::Data>,
    images: Vec<::gltf::image::Data>,
}

impl<'catalogs> Importer<'catalogs> {
    fn import(&mut self, document: ::gltf::Document) -> Vec<ImportGltfError> {
        let mut errors = vec![];
        let scene = document.default_scene().unwrap();
        for node in scene.nodes() {
            if let Err(e) = self.import_gltf_node(node, None) {
                errors.push(e);
            }
        }
        errors
    }

    fn import_gltf_texture(
        &mut self,
        texture: gltf::Texture,
    ) -> Result<Id<Texture>, ImportGltfError> {
        if let gltf::image::Source::Uri { uri, .. } = texture.source().source() {
            let data = &self
                .images
                .get(texture.source().index())
                .ok_or(ImportGltfError::ImageFileNotFound(uri.to_string()))?;
            let loaded = image::load_from_memory(&data.pixels)
                .map_err(|_| ImportGltfError::ImageLoadingFailed(uri.to_string()))?;
            let width = data.width;
            let height = data.height;
            let rgba = loaded.into_rgba();
            let bytes = rgba.as_bytes();
            Ok(self
                .textures
                .add(Texture::from_rgba(&bytes, width as i32, height as i32)))
        } else {
            unimplemented!("Only relative uri image loading is implemented")
        }
    }

    fn import_gltf_material(
        &mut self,
        material: gltf::Material,
    ) -> Result<Id<Material>, ImportGltfError> {
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
        Ok(self.materials.add(Material {
            normal,
            diffuse,
            base_diffuse_color,
        }))
    }

    fn import_gltf_mesh(&mut self, mesh: gltf::Mesh) -> Result<Id<Mesh>, ImportGltfError> {
        let mut primitives = vec![];
        for primitive in mesh.primitives() {
            let reader = primitive
                .reader(|buffer| self.buffers.get(buffer.index()).map(|it| it.0.as_slice()));

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

            primitives.push(Primitive::new(
                &positions, &normals, &uvs, &indices, material,
            ));
        }
        Ok(self.meshes.add(Mesh { primitives }))
    }

    fn import_gltf_node(
        &mut self,
        node: gltf::Node,
        parent: Option<Id<Entity>>,
    ) -> Result<Id<Entity>, ImportGltfError> {
        let mesh = match node.mesh() {
            Some(mesh) => Some(self.import_gltf_mesh(mesh)?),
            None => None,
        };
        let entity = Entity {
            children: vec![],
            parent,
            mesh,
            transform: node.transform().matrix(),
        };
        let id = self.entities.add(entity);

        let mut children = vec![];
        for child in node.children() {
            let child_id = self.import_gltf_node(child, Some(id))?;
            children.push(child_id);
        }
        self.entities
            .get_mut(id)
            .ok_or(ImportGltfError::Unreachable)?
            .children = children;

        Ok(id)
    }
}

#[derive(thiserror::Error, Debug)]
pub enum ImportGltfError {
    #[error("image file '{0}' not found.")]
    ImageFileNotFound(String),
    #[error("image loading failed for file '{0}'")]
    ImageLoadingFailed(String),
    #[error(
        "required property '{0}' is missing for mesh with index {1} and primitive with index {2}"
    )]
    RequiredMeshPropertyMissing(&'static str, usize, usize),
    #[error("unreachable")]
    Unreachable,
}
