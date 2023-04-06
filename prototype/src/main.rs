
use std::env::args;
use image::{ImageBuffer, Rgba, Pixel};
use std::env;
use std::error::Error;
use std::fs::File;
use std::io::{Read, Write};
use std::process::{Command, Stdio};
use gtk::{Application};
use gio::prelude::*;
use gtk::prelude::*;

mod gui;

fn main() {

    // GUI 
    let application = Application::new(Some("com.example.video_uploader"), Default::default())
    .expect("Failed to initialize GTK application");

    application.connect_activate(|app| {
        gui::build_ui(app);
    });

    application.run(&args().collect::<Vec<_>>());

    // ENCODER

    // Read the input file into a Vec<u8>
    let mut input_file = File::open("testfiles/enwik9").unwrap();
    let mut input_data = Vec::new();
    input_file.read_to_end(&mut input_data).unwrap();

    // Calculate the number of frames we need to create
    let frame_size = 1920 * 1080 * 4;
    let num_frames = (input_data.len() + frame_size - 1) / frame_size;

    // Create a new RGBA image buffer for each frame
    let mut frames = Vec::new();
    for _ in 0..num_frames {
        frames.push(ImageBuffer::<Rgba<u8>, Vec<u8>>::new(1920, 1080));
    }

    // Split the input data into chunks of 1920 x 1080 x 4 bytes and
    // fill each frame with the corresponding chunk of data
    for (i, chunk) in input_data.chunks(frame_size).enumerate() {
        let frame = &mut frames[i];
        for (j, pixel) in chunk.chunks_exact(4).enumerate() {
            let x = j % 1920;
            let y = j / 1920;
            let pixel = Rgba::from_slice(pixel);
            frame.put_pixel(x as u32, y as u32, *pixel);
        }
    }

    // Save each frame as a PNG image file
    for (i, frame) in frames.iter().enumerate() {
        let filename = format!("frames/frame{}.png", i);
        frame.save(&filename).unwrap();
    }

    // Convert the PNG frames to an MP4 video using FFmpeg
    let output_filename = "output/output.mp4";
    let mut ffmpeg = Command::new("C:/ffmpeg/bin/ffmpeg.exe")
        .args(&[
            "-y", // overwrite output file if it exists
            "-f",
            "image2pipe",
            "-vcodec",
            "png",
            "-r",
            "60",
            "-i",
            "-", // read from stdin
            "-vcodec",
            "libx264",
            "-pix_fmt",
            "yuv420p",
            "-preset",
            "veryslow",
            "-crf",
            "17",
            "-r",
            "60",
            output_filename,
        ])
        .stdin(Stdio::piped())
        .arg("-loglevel")
        .arg("error")
        .spawn()
        .unwrap();
    let mut stdin = ffmpeg.stdin.take().unwrap();
    for (i, frame) in frames.iter().enumerate() {
        let filename = format!("frames/frame{}.png", i);
        let mut file = File::open(&filename).unwrap();
        let mut data = Vec::new();
        file.read_to_end(&mut data).unwrap();
        stdin.write_all(&data).unwrap();
    }
    drop(stdin);
    ffmpeg.wait().unwrap();
}
