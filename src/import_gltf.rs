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
) {
    let (document, buffers, images) = ::gltf::import(file_name).unwrap();
    let mut importer = Importer {
        entities,
        meshes,
        materials,
        textures,
        buffers,
        images,
    };
    importer.import(document);
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
    fn import(&mut self, document: ::gltf::Document) {
        let scene = document.default_scene().unwrap();
        for node in scene.nodes() {
            self.import_gltf_node(node, None);
        }
    }

    fn import_gltf_texture(&mut self, texture: gltf::Texture) -> Id<Texture> {
        if let gltf::image::Source::Uri { .. } = texture.source().source() {
            let data = &self.images[texture.source().index()];
            let loaded = image::load_from_memory(&data.pixels).unwrap();
            let width = data.width;
            let height = data.height;
            let rgba = loaded.into_rgba();
            let bytes = rgba.as_bytes();
            self.textures
                .add(Texture::from_rgba(&bytes, width as i32, height as i32))
        } else {
            unimplemented!("Only relative uri image loading is implemented")
        }
    }

    fn import_gltf_material(&mut self, material: gltf::Material) -> Id<Material> {
        let normal = material
            .normal_texture()
            .as_ref()
            .map(|info| self.import_gltf_texture(info.texture()));
        let diffuse = material
            .pbr_metallic_roughness()
            .base_color_texture()
            .as_ref()
            .map(|info| self.import_gltf_texture(info.texture()));
        let base_diffuse_color = material.pbr_metallic_roughness().base_color_factor();
        self.materials.add(Material {
            normal,
            diffuse,
            base_diffuse_color,
        })
    }

    fn import_gltf_mesh(&mut self, mesh: gltf::Mesh) -> Id<Mesh> {
        let primitives = mesh
            .primitives()
            .map(|primitive| {
                let reader = primitive
                    .reader(|buffer| self.buffers.get(buffer.index()).map(|it| it.0.as_slice()));

                let positions = reader.read_positions().unwrap().collect::<Vec<_>>();
                let normals = reader.read_normals().unwrap().collect::<Vec<_>>();
                let uvs = reader
                    .read_tex_coords(0)
                    .unwrap()
                    .into_f32()
                    .collect::<Vec<_>>();
                let indices = reader
                    .read_indices()
                    .unwrap()
                    .into_u32()
                    .map(|it| it as u16) // TODO! this sucks
                    .collect::<Vec<_>>();

                let material = self.import_gltf_material(primitive.material());

                Primitive::new(&positions, &normals, &uvs, &indices, material)
            })
            .collect();
        self.meshes.add(Mesh { primitives })
    }

    fn import_gltf_node(&mut self, node: gltf::Node, parent: Option<Id<Entity>>) -> Id<Entity> {
        let entity = Entity {
            children: vec![],
            parent,
            mesh: node.mesh().map(|mesh| self.import_gltf_mesh(mesh)),
            transform: node.transform().matrix(),
        };
        let id = self.entities.add(entity);

        let children = node
            .children()
            .map(|child| self.import_gltf_node(child, Some(id)))
            .collect();
        self.entities.get_mut(id).unwrap().children = children;

        id
    }
}
