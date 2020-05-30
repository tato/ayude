#[derive(Debug)]
pub struct Geometry {
    vao: u32,
}

impl Geometry {
    pub fn new(positions: &[[f32; 3]], normals: &[[f32; 3]], uvs: &[[f32; 2]], indices: &[u16]) -> Self {
        assert!(positions.len() == normals.len() && positions.len() == uvs.len(),
            "There are different amounts of components for this Geometry\npositions[{}], normals[{}], uvs[{}]",
            positions.len(), normals.len(), uvs.len());

        unsafe {
            let mut vao = 0;
            gl::GenVertexArrays(1, &mut vao);
            gl::BindVertexArray(vao);

            let mut position_buffer = 0;
            gl::GenBuffers(1, &mut position_buffer);
            gl::BindBuffer(gl::ARRAY_BUFFER, position_buffer);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (std::mem::size_of::<[f32; 3]>() * positions.len()) as isize,
                positions.as_ptr() as *const std::ffi::c_void,
                gl::STATIC_DRAW
            );
            gl::EnableVertexAttribArray(0);
            gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, 0, std::ptr::null());

            let mut normal_buffer = 0;
            gl::GenBuffers(1, &mut normal_buffer);
            gl::BindBuffer(gl::ARRAY_BUFFER, normal_buffer);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (std::mem::size_of::<[f32; 3]>() * normals.len()) as isize,
                normals.as_ptr() as *const std::ffi::c_void,
                gl::STATIC_DRAW
            );
            gl::EnableVertexAttribArray(1);
            gl::VertexAttribPointer(1, 3, gl::FLOAT, gl::FALSE, 0, std::ptr::null());

            let mut uv_buffer = 0;
            gl::GenBuffers(1, &mut uv_buffer);
            gl::BindBuffer(gl::ARRAY_BUFFER, uv_buffer);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (std::mem::size_of::<[f32; 2]>() * uvs.len()) as isize,
                uvs.as_ptr() as *const std::ffi::c_void,
                gl::STATIC_DRAW
            );
            gl::EnableVertexAttribArray(2);
            gl::VertexAttribPointer(2, 2, gl::FLOAT, gl::FALSE, 0, std::ptr::null());

            let mut ebo = 0;
            gl::GenBuffers(1, &mut ebo);
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ebo);
            gl::BufferData(
                gl::ELEMENT_ARRAY_BUFFER,
                (std::mem::size_of::<u16>() * indices.len()) as isize,
                indices.as_ptr() as *const std::ffi::c_void,
                gl::STATIC_DRAW
            );

            gl::BindVertexArray(0);
            gl::BindBuffer(gl::ARRAY_BUFFER, 0);
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, 0);

            Geometry{ vao }
        }
    }
}

impl Drop for Geometry {
    fn drop(&mut self) {
        unsafe {
            // glBindVertexArray(Mesh->MeshID);
            //
            // int32 BufferIDs[4];
            // glGetVertexAttribiv(0, GL_VERTEX_ATTRIB_ARRAY_BUFFER_BINDING, BufferIDs);
            // glGetVertexAttribiv(1, GL_VERTEX_ATTRIB_ARRAY_BUFFER_BINDING, BufferIDs+1);
            // glGetVertexAttribiv(2, GL_VERTEX_ATTRIB_ARRAY_BUFFER_BINDING, BufferIDs+2);
            // glGetIntegerv(GL_ELEMENT_ARRAY_BUFFER_BINDING,                BufferIDs+3);
            // glDeleteBuffers(4, (uint32 *)BufferIDs);
            //
            // glBindVertexArray(0);

            gl::BindVertexArray(self.vao);

            gl::BindVertexArray(0);
            todo!();
        }
    }
}