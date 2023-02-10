extern crate gl;
extern crate glfw;
mod utils;
use glfw::Context;
use libpulse_binding::sample::{Format, Spec};
use libpulse_binding::stream::Direction;
use libpulse_simple_binding::Simple;
use spectrum_analyzer::scaling::divide_by_N;
use spectrum_analyzer::windows::hann_window;
use spectrum_analyzer::{samples_fft_to_spectrum, FrequencyLimit};
use std::f32::INFINITY;
use std::mem::*;
fn main() {
    let spec = Spec {
        format: Format::S16le,
        channels: 1,
        rate: 44100,
    };
    assert!(spec.is_valid());
    let s = Simple::new(
        None,
        "audiofft-rs",
        Direction::Record,
        None,
        "visualizer",
        &spec,
        None,
        None,
    )
    .unwrap();
    let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();

    let (mut window, _events) = glfw
        .create_window(1366, 768, "audiofft-rs", glfw::WindowMode::Windowed)
        .expect("Failed to create GLFW window.");
    window.make_current();
    window.set_key_polling(true);

    gl::load_with(|s| window.get_proc_address(s));
    gl::Viewport::load_with(|s| window.get_proc_address(s));
    utils::compile_shaders();
    let mut last_buffer: Vec<f32> = vec![0.0; (utils::N + 2) as usize];
    while !window.should_close() {
        let mut buf1: Vec<u8> = vec![0; (utils::N * 16) as usize];
        s.read(&mut buf1).unwrap();
        let buf2: Vec<f32> = buf1.iter().map(|&x| x as f32).collect();
        let hann_window = hann_window(&buf2);
        let spectrum_hann_window = samples_fft_to_spectrum(
            &hann_window,
            44100,
            FrequencyLimit::Max(15000.0),
            Some(&divide_by_N),
        )
        .unwrap();
        let mut buffer: Vec<f32> = spectrum_hann_window
            .data()
            .iter()
            .map(|x| x.1.val())
            .collect();
        buffer = buffer[0..=(utils::N + 1) as usize].to_vec();
        let mut max: f32 = -INFINITY;
        let mut min: f32 = INFINITY;
        let smooth_const_up = 0.8;
        let smooth_const_down = 0.2;
        for i in 0..buffer.len() {
            if buffer[i] < last_buffer[i] {
                buffer[i] =
                    last_buffer[i] * smooth_const_down + buffer[i] * (1.0 - smooth_const_down);
            } else {
                buffer[i] = last_buffer[i] * smooth_const_up + buffer[i] * (1.0 - smooth_const_up);
            }
            if buffer[i] > max {
                max = buffer[i];
            }
            if buffer[i] < min {
                min = buffer[i];
            }
        }
        last_buffer = buffer.clone();
        let gap = max - min;
        let mut height: Vec<f32> = Vec::new();
        for i in 1..buffer.len() {
            height.push(((buffer[i] - min) / gap) * 2.0);
        }
        let vertices = utils::compute_bar_vertice(&height);
        let indices = utils::compute_bar_indices();
        // Rendering goes here
        let (vao, vbo, ebo) = utils::init_objects();
        unsafe {
            gl::BindVertexArray(vao);
            gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                size_of_val(&vertices) as isize,
                vertices.as_ptr().cast(),
                gl::STATIC_DRAW,
            );

            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ebo);
            gl::BufferData(
                gl::ELEMENT_ARRAY_BUFFER,
                size_of_val(&indices) as isize,
                indices.as_ptr().cast(),
                gl::STATIC_DRAW,
            );
            utils::link_attributes();
            gl::Clear(gl::COLOR_BUFFER_BIT);
            gl::DrawElements(
                gl::TRIANGLES,
                (utils::N + 1) * 6,
                gl::UNSIGNED_INT,
                0 as *const _,
            );
            // count: num of indices
        }

        window.swap_buffers();
        glfw.poll_events();
    }
}
