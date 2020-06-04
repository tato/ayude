
fn main() {
    let scene = ayude::gltf::load_gltf("samples/glTF-Sample-Models/2.0/Sponza/glTF/Sponza.gltf").unwrap();
    let ser = bincode::serialize(&scene).unwrap();
    let mut f = std::fs::File::create("jaja.dat").unwrap();
    use std::io::Write;
    f.write(&ser).unwrap();
}