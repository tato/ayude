
pub fn load_blend(file_name: &str) -> Result<ayude::gltf::UnloadedScene, ayude::AyudeError> {
    let blend = blend::Blend::from_file(file_name);
    
    todo!()
}

fn main() {
    let scene = ayude::gltf::load_gltf("samples/glTF-Sample-Models/2.0/Sponza/glTF/Sponza.gltf").unwrap();

    // assert!(cfg!(target_os = "windows"));
    // let output = std::process::Command::new("crunch.exe")
    //     .output()
    //     .unwrap();
    // println!("{:?}", output);


    let mut f = std::fs::File::create("jaja.dat").unwrap();
    let start = std::time::Instant::now();
    bincode::serialize_into(&mut f, &scene).unwrap();
    println!("{:?}", start.elapsed());
}