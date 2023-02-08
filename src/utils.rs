use std::mem::*;
use std::ptr::null;
const vertexShaderSource: &'static str = "
#version 330 core

layout (location = 0) in vec3 pos;

void main() {
    gl_Position = vec4(pos.x, pos.y, pos.z, 1.0);
}";
const fragmentShaderSource: &'static str = "
#version 330 core
out vec4 FragColor;

void main() {
    FragColor = vec4(1.0f, 1.0f, 1.0f, 1.0f);
} 
";
pub type Vertex = [f32; 3];
pub type TriIndexes = [u32; 3];
pub fn compile_shaders() {
    unsafe {
        let vertexShader = gl::CreateShader(gl::VERTEX_SHADER);
        let v_str = std::ffi::CString::new(vertexShaderSource.as_bytes()).unwrap();
        gl::ShaderSource(vertexShader, 1, &v_str.as_ptr(), null());
        gl::CompileShader(vertexShader);
        let fragmentShader = gl::CreateShader(gl::FRAGMENT_SHADER);
        let f_str = std::ffi::CString::new(fragmentShaderSource.as_bytes()).unwrap();
        gl::ShaderSource(fragmentShader, 1, &f_str.as_ptr(), null());
        gl::CompileShader(fragmentShader);
        let shaderProgram = gl::CreateProgram();
        // I think I don't need to check compile errors... for now
        gl::AttachShader(shaderProgram, vertexShader);
        gl::AttachShader(shaderProgram, fragmentShader);
        gl::LinkProgram(shaderProgram);
        gl::UseProgram(shaderProgram);
        gl::DeleteShader(vertexShader);
        gl::DeleteShader(fragmentShader);
    }
}
pub fn init_objects() -> (u32, u32, u32) {
    let mut vao = 0;
    let mut vbo = 0;
    let mut ebo = 0;
    unsafe {
        gl::GenVertexArrays(1, &mut vao);
        gl::GenBuffers(1, &mut vbo);
        gl::GenBuffers(1, &mut ebo);
        assert_ne!(vao, 0);
        assert_ne!(vbo, 0);
        assert_ne!(ebo, 0);
    }
    (vao, vbo, ebo)
}
pub fn link_attributes() {
    unsafe {
        gl::VertexAttribPointer(
            0,
            3,
            gl::FLOAT,
            gl::FALSE,
            size_of::<Vertex>().try_into().unwrap(),
            0 as *const _,
        );
        gl::EnableVertexAttribArray(0);
    }
}
pub const N: i32 = 1024;
pub fn compute_bar_vertice(height: &[f32]) -> [Vertex; (N * 3 + 1) as usize] {
    let l_unit = 2.0 / (N as f32);
    let mut res: Vec<Vertex> = Vec::new();
    res.push([-1.0, -1.0, 0.0]);

    for _i in 1..=N {
        res.push([res.last().unwrap()[0] + l_unit, -1.0, 0.0]);
    }
    res.push([-1.0, height[0] - 1.0, 0.0]);
    for i in 1..(N * 2) {
        res.push([
            res[(((i + 1) as f32) / 2.0).floor() as usize][0],
            height[((i as f32) / 2.0).floor() as usize] - 1.0,
            0.0,
        ]);
    }
    // println!("{:?}", res);
    res.as_slice()
        .try_into()
        .expect("slice with incorrect length")
}
pub fn compute_bar_indices() -> [TriIndexes; (N * 2) as usize] {
    let mut res: Vec<TriIndexes> = Vec::new();
    let mid: u32 = (N + 1) as u32;
    for i in 0..N {
        let j: u32 = i as u32;
        res.push([j, j + 1, mid + j * 2]);
        res.push([j + 1, mid + j * 2, mid + j * 2 + 1]);
    }
    // println!("{:?}", res);
    res.as_slice()
        .try_into()
        .expect("slice with incorrect length")
}
