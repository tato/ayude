use crate::graphics::GraphicsError;

#[derive(Debug)]
pub struct Shader {
    id: u32,
}

impl Shader {
    pub fn from_sources(vertex: &str, fragment: &str) -> Result<Shader, GraphicsError> {
        let vertex_id = create_single_shader_from_source(vertex.as_bytes(), gl::VERTEX_SHADER)?;
        let fragment_id = create_single_shader_from_source(fragment.as_bytes(), gl::FRAGMENT_SHADER)?;

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

                if info_log_length > 0 {
                    let mut info_log = vec![0u8; info_log_length as usize];
                    gl::GetProgramInfoLog(program_id, info_log_length-1, std::ptr::null_mut(), (&mut info_log).as_mut_ptr() as *mut i8);

                    let error = std::ffi::CStr::from_bytes_with_nul(&info_log)?.to_str()?.to_string();
                    Err(error.into())
                } else {
                    Err("Program didn't compile and it didn't provide an info log".into())
                }
            } else {
                /*
                glGetProgramiv(program, GL_ACTIVE_UNIFORMS, &count);
                printf("Active Uniforms: %d\n", count);

                for (i = 0; i < count; i++)
                {
                    glGetActiveUniform(program, (GLuint)i, bufSize, &length, &size, &type, name);

                    printf("Uniform #%d Type: %u Name: %s\n", i, type, name);
                }
                */
                let mut count = 0;
                gl::GetProgramiv(program_id, gl::ACTIVE_UNIFORMS, &mut count);

                let mut buffer = [0u8; 256];
                for i in 0..count {
                    let mut result_length = 0;
                    let mut result_size = 0;
                    let mut result_type = 0;
                    gl::GetActiveUniform(program_id, i as u32, buffer.len() as i32, &mut result_length, &mut result_size, &mut result_type, buffer.as_mut_ptr() as *mut i8);

                    let name = std::str::from_utf8_unchecked(&buffer[0..result_length as usize]);
                    println!("{}: {} ({})", name, result_type, result_size);
                }
                Ok(Shader { id: program_id })
            }
        }
    }
}

impl Drop for Shader {
    fn drop(&mut self) {
        unsafe { gl::DeleteProgram(self.id); }
    }
}

fn create_single_shader_from_source(source: &[u8], shader_type: u32) -> Result<u32, GraphicsError> {
    unsafe {
        let shader_id = gl::CreateShader(shader_type);
        gl::ShaderSource(shader_id, 1,  &source.as_ptr() as *const *const u8 as *const *const i8, &source.len() as *const usize as *const i32);
        gl::CompileShader(shader_id);

        let mut shader_compilation_result = gl::FALSE as i32;
        gl::GetShaderiv(shader_id, gl::COMPILE_STATUS, &mut shader_compilation_result);

        if shader_compilation_result == gl::TRUE as i32 {
            Ok(shader_id)
        } else {
            let mut info_log_length = 0;
            gl::GetShaderiv(shader_id, gl::INFO_LOG_LENGTH, &mut info_log_length);

            if info_log_length > 0 {
                let mut info_log = vec![0u8; info_log_length as usize];
                gl::GetShaderInfoLog(shader_id, info_log_length-1, std::ptr::null_mut(), (&mut info_log).as_mut_ptr() as *mut i8);
                let error = std::ffi::CStr::from_bytes_with_nul(&info_log)?.to_str()?.to_string();
                Err(error.into())
            } else {
                Err("Program didn't compile and it didn't provide an info log".into())
            }
        }
    }
}