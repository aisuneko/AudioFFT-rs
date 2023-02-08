#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]

extern crate gl;
extern crate glfw;
mod utils;
use std::error::Error;
use std::f32::consts::PI;
use std::f32::INFINITY;
use std::mem::*;
// use gl::types::*;
use glfw::{Action, Context, Key};
use libpulse_binding::error::PAErr;
use libpulse_binding::sample::{Format, Spec};
use libpulse_binding::stream::Direction;
use libpulse_simple_binding::Simple;
use spectrum_analyzer::scaling::divide_by_N;
use spectrum_analyzer::windows::hann_window;
use spectrum_analyzer::{samples_fft_to_spectrum, Frequency, FrequencyLimit, FrequencyValue};

// use rustfft::{num_complex::Complex, FftPlanner};

// const HEIGHT: [f32; utils::N as usize] = [0.25, 0.3, 0.5, 1.0, 0.2, 0.4, 1.5, 1.2]; // maximum: 2
fn main() {
    let spec = Spec {
        format: Format::S16le,
        channels: 1,
        rate: 44100,
    };
    assert!(spec.is_valid());
    let s = Simple::new(
        None,              // Use the default server
        "audvisdemo-rs",   // Our application’s name
        Direction::Record, // We want a playback stream
        None,              // Use the default device
        "visualizer",      // Description of our stream
        &spec,             // Our sample format
        None,              // Use default channel map
        None,              // Use default buffering attributes
    )
    .unwrap();
    let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();

    // Create a windowed mode window and its OpenGL context
    let (mut window, events) = glfw
        .create_window(
            1000,
            500,
            "Hello this is window",
            glfw::WindowMode::Windowed,
        )
        .expect("Failed to create GLFW window.");
    // Make the window's context current
    window.make_current();
    window.set_key_polling(true);

    gl::load_with(|s| window.get_proc_address(s));
    gl::Viewport::load_with(|s| window.get_proc_address(s));
    unsafe {
        utils::compile_shaders();
    }
    let mut last_buffer: Vec<f32> = vec![0.0; (utils::N + 1) as usize];
    // Loop until the user closes the window
    while !window.should_close() {
        let mut buf1: Vec<u8> = vec![0; (utils::N * 2) as usize];
        s.read(&mut buf1).unwrap();
        let mut buf2: Vec<f32> = Vec::new();
        for i in 0..buf1.len() {
            // buf2.push(
            //     (0.5 * (1.0
            //         - (2.0 * (-1.0 as f32).acos() * (i as f32) / (utils::N - 1) as f32).cos()))
            //         * buf1[i] as f32,
            // );
            buf2.push(
                0.5 * (2.0 * PI * (utils::N as f32) / (utils::N - 1) as f32) * buf1[i] as f32,
            );
        }

        let mut buf3: [f32; (utils::N * 2) as usize] = buf2
            .as_slice()
            .try_into()
            .expect("slice with incorrect length");
        let hann_window = hann_window(&buf3);
        let spectrum_hann_window = samples_fft_to_spectrum(
            // (windowed) samples
            &hann_window,
            // sampling rate
            44100,
            // optional frequency limit: e.g. only interested in frequencies 50 <= f <= 150?
            FrequencyLimit::All,
            // optional scale
            Some(&divide_by_N),
        )
        .unwrap();
        let mut buf4: Vec<(Frequency, FrequencyValue)> = spectrum_hann_window.data().to_vec();
        buf4.sort_by_key(|x| x.0);
        let mut buffer: Vec<f32> = buf4.iter().map(|x| x.1.val()).collect();

        // println!("{:?}", buf1);
        // println!("{:?}", buffer);
        let mut max: f32 = -INFINITY;
        let mut min: f32 = INFINITY;
        let mut smooth_const_up = 0.8;
        let mut smooth_const_down = 0.2;
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
        let mut res: Vec<f32> = Vec::new();
        for i in 0..buffer.len() {
            res.push(((buffer[i] - min) / gap) * 2.0);
        }
        let height: [f32; (utils::N + 1) as usize] = res
            .as_slice()
            .try_into()
            .expect("slice with incorrect length");
        let vertices = utils::compute_bar_vertice(&height);
        let indices = utils::compute_bar_indices();
        // std::thread::sleep(std::time::Duration::from_millis(500));
        for (_, event) in glfw::flush_messages(&events) {
            // println!("{:?}", event);
            match event {
                glfw::WindowEvent::Key(Key::Escape, _, Action::Press, _) => {
                    window.set_should_close(true)
                }
                _ => {}
            }
        }
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
            // gl::ClearColor(0.3, 0.3, 0.3, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);
            gl::DrawElements(
                gl::TRIANGLES,
                (utils::N + 1) * 6,
                gl::UNSIGNED_INT,
                0 as *const _,
            );
            // count: num of indices
        }
        // Swap front and back buffers
        window.swap_buffers();

        // Poll for and process events
        glfw.poll_events();
    }
}
