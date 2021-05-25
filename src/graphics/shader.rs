use std::collections::HashMap;

pub struct Shader {
    id: u32,
    // todo! uniform_locations: HashMap<&'static str, i32>,
    uniforms: HashMap<&'static str, Box<dyn Uniform>>,
}

impl Shader {
    pub fn from_sources(vertex: &str, fragment: &str) -> Result<Shader, ShaderError> {
        let vertex_id = create_single_shader_from_source(vertex.as_bytes(), gl::VERTEX_SHADER)?;
        let fragment_id =
            create_single_shader_from_source(fragment.as_bytes(), gl::FRAGMENT_SHADER)?;

        unsafe {
            let program_id = gl::CreateProgram();

            gl::AttachShader(program_id, vertex_id);
            gl::AttachShader(program_id, fragment_id);
            gl::LinkProgram(program_id);

            gl::DetachShader(program_id, vertex_id);
            gl::DetachShader(program_id, fragment_id);

            gl::DeleteShader(vertex_id);
            gl::DeleteShader(fragment_id);

            let mut program_compilation_was_ok: i32 = gl::FALSE as i32;
            gl::GetProgramiv(program_id, gl::LINK_STATUS, &mut program_compilation_was_ok);

            if program_compilation_was_ok == gl::FALSE as i32 {
                let mut info_log_length = 0;
                gl::GetProgramiv(program_id, gl::INFO_LOG_LENGTH, &mut info_log_length);

                let program_source_string = format!("VERTEX:\n{}\nFRAGMENT:\n{}", vertex, fragment);

                if info_log_length > 0 {
                    let mut info_log = vec![0u8; info_log_length as usize];
                    gl::GetProgramInfoLog(
                        program_id,
                        info_log_length - 1,
                        std::ptr::null_mut(),
                        (&mut info_log).as_mut_ptr() as *mut i8,
                    );

                    let error = std::ffi::CStr::from_bytes_with_nul(&info_log)
                        .ok()
                        .and_then(|s| s.to_str().ok())
                        .unwrap_or(
                            "unknown error: info log had nul byte or non valid utf8 character",
                        )
                        .to_string();

                    Err(ShaderError::FailedCompile(error, program_source_string))
                } else {
                    Err(ShaderError::FailedCompile(
                        "program didn't compile and it didn't provide an info log".to_string(),
                        program_source_string,
                    ))
                }
            } else {
                let mut count = 0;
                gl::GetProgramiv(program_id, gl::ACTIVE_UNIFORMS, &mut count);

                let mut buffer = [0u8; 256];
                for i in 0..count {
                    let mut result_length = 0;
                    let mut result_size = 0;
                    let mut result_type = 0;
                    gl::GetActiveUniform(
                        program_id,
                        i as u32,
                        buffer.len() as i32,
                        &mut result_length,
                        &mut result_size,
                        &mut result_type,
                        buffer.as_mut_ptr() as *mut i8,
                    );

                    // let name = std::str::from_utf8_unchecked(&buffer[0..result_length as usize]);
                    // println!("{}: {} ({})", name, result_type, result_size);
                }
                Ok(Shader {
                    id: program_id,
                    uniforms: HashMap::new(),
                })
            }
        }
    }

    pub fn uniform(&mut self, name: &'static str, value: impl Uniform + 'static) {
        self.uniforms.insert(name, Box::new(value));
    }

    pub fn bind(&self) {
        unsafe {
            gl::UseProgram(self.id);
            let mut texture_offset = 0;
            for (&name, value) in &self.uniforms {
                let location =
                    gl::GetUniformLocation(self.id, format!("{}\0", name).as_ptr() as *const i8);
                value.bind(location, &mut texture_offset);
            }
        }
    }
}

impl Drop for Shader {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteProgram(self.id);
        }
    }
}

pub trait Uniform {
    unsafe fn bind(&self, location: i32, texture_offset: &mut u32);
}
impl Uniform for bool {
    unsafe fn bind(&self, location: i32, _: &mut u32) {
        gl::Uniform1i(location, *self as i32);
    }
}
impl Uniform for [f32; 3] {
    unsafe fn bind(&self, location: i32, _: &mut u32) {
        gl::Uniform3f(location, self[0], self[1], self[2]);
    }
}
impl Uniform for [f32; 4] {
    unsafe fn bind(&self, location: i32, _: &mut u32) {
        gl::Uniform4f(location, self[0], self[1], self[2], self[3]);
    }
}
impl Uniform for [[f32; 4]; 4] {
    unsafe fn bind(&self, location: i32, _: &mut u32) {
        gl::UniformMatrix4fv(location, 1, gl::FALSE, self.as_ptr() as *const f32);
    }
}
impl Uniform for crate::graphics::Texture {
    unsafe fn bind(&self, location: i32, texture_offset: &mut u32) {
        gl::ActiveTexture(gl::TEXTURE0 + *texture_offset);
        gl::BindTexture(gl::TEXTURE_2D, *self.id);
        gl::Uniform1i(location, *texture_offset as i32);
        *texture_offset += 1;
    }
}

fn create_single_shader_from_source(source: &[u8], shader_type: u32) -> Result<u32, ShaderError> {
    unsafe {
        let shader_id = gl::CreateShader(shader_type);
        gl::ShaderSource(
            shader_id,
            1,
            &source.as_ptr() as *const *const u8 as *const *const i8,
            &source.len() as *const usize as *const i32,
        );
        gl::CompileShader(shader_id);

        let mut shader_compilation_result = gl::FALSE as i32;
        gl::GetShaderiv(
            shader_id,
            gl::COMPILE_STATUS,
            &mut shader_compilation_result,
        );

        if shader_compilation_result == gl::TRUE as i32 {
            Ok(shader_id)
        } else {
            let mut info_log_length = 0;
            gl::GetShaderiv(shader_id, gl::INFO_LOG_LENGTH, &mut info_log_length);

            let source_string = std::str::from_utf8(source)
                .unwrap_or("invalid source slice")
                .to_string();

            if info_log_length > 0 {
                let mut info_log = vec![0u8; info_log_length as usize];
                gl::GetShaderInfoLog(
                    shader_id,
                    info_log_length - 1,
                    std::ptr::null_mut(),
                    (&mut info_log).as_mut_ptr() as *mut i8,
                );
                let error = std::ffi::CStr::from_bytes_with_nul(&info_log)
                    .ok()
                    .and_then(|s| s.to_str().ok())
                    .unwrap_or("unknown error: info log had nul byte or non valid utf8 character")
                    .to_string();
                Err(ShaderError::FailedCompile(error, source_string))
            } else {
                Err(ShaderError::FailedCompile(
                    "program didn't compile and it didn't provide an info log".to_string(),
                    source_string,
                ))
            }
        }
    }
}

#[derive(thiserror::Error, Debug)]
pub enum ShaderError {
    #[error("shader failed to compile with error: '{0}'\nshader source is: \n{1}")]
    FailedCompile(String, String),
}
