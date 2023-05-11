use image::{ImageBuffer, Rgb};

use std::fs::File;
use std::io::{Read, Write, Seek};
use std::process::Command;
use std::sync::{Arc, Mutex};

use rayon::prelude::*;

use crate::backend::file::FileInfo;

///A struct holding the essential parts to the encoder such as file, res and pixel size.
pub struct Encode {
    file: FileInfo,
    res: (usize, usize),
    square_w: usize,
    square_h: usize,
}

impl Encode {

    /// Constructs a new `Encode`.
    ///
    /// # Arguments
    ///
    /// * `file` - Contains the file data.
    /// * `res` - The video resolution.
    /// * `sqaure_w` - The pixels width.
    /// * `square_h` - The pixels height.
    pub fn new(file: FileInfo, res: (usize, usize), square_w: usize, square_h: usize) -> Encode {
        Encode {
            file,
            res,
            square_w,
            square_h,
        }
    }

    /// Encodes the specified file into a mp4.
    ///
    /// # Arguments
    ///
    /// * `Encode` - The Encode struct containing the neccesary parts for the encoding.
    /// 
    /// # Returns
    /// 
    /// A String representing the mp4 output path.
    pub fn encoder(encode: Encode) -> String {
   
        //Get file size
        let file_size = FileInfo::size(&encode.file);
    
        //Calculate number of frames needed to contain every bit from the specified file
        let frame_size: usize = (encode.res.0 * encode.res.1) / (encode.square_w * encode.square_h);
        let num_bits = file_size * 8;
        let num_frames = (num_bits + frame_size  - 1) / frame_size;
    
        //Create the metadata for the mp4
        let title = &format!("title={}", FileInfo::name(&encode.file));
        let datatype = &format!("author={}", FileInfo::datatype(&encode.file));
        let date = &format!("time={}", FileInfo::date(&encode.file));
        let output = &format!("../videos/{}.mp4", FileInfo::name(&encode.file));
    
        // Convert the frames to an MP4 video using FFmpeg
        let ffmpeg = Command::new("ffmpeg")
            .args(&[
                "-y",
                "-framerate", "30",
                "-f", "rawvideo",
                "-pix_fmt", "rgb24",
                "-s", &format!("{}x{}", encode.res.0,  encode.res.1),
                "-i", "-",
                "-c:v", "libx264",
                "-crf", "18",
                "-preset", "ultrafast",
                "-b:v", "1000M",
                "-maxrate", "1000M",
                "-bufsize", "1000M",
                "-movflags", "+faststart",
                "-map_metadata", "0",
                "-metadata",  title,
                "-metadata", datatype,
                "-metadata", date,
                output,
            ])
            .stdin(std::process::Stdio::piped())
            .spawn()
            .unwrap();
            
            //Arc mutex is needed so that every thread has access to the ffmpeg process
            let ffmpeg_process = Arc::new(Mutex::new(ffmpeg));
            
            //Proccessing every frame using paralellism
            (0..num_frames).into_par_iter().for_each(|frame_index| {
                //create a ImageBuffer to store the pixels
                let mut frame = ImageBuffer::<Rgb<u8>, _>::new(encode.res.0 as u32, encode.res.1 as u32);
    
                //Calculate the start and end of the chunk of data 
                let start = frame_index * frame_size / 8;
                let end = std::cmp::min(start + frame_size / 8, file_size);
    
                //Creates a buffer for the chunk, seek to the specified start place and read in the input data to the chunck Vec.
                let mut chunk = vec![0u8; (end - start) as usize ];
                let mut input_file = File::open(FileInfo::path(&encode.file)).unwrap();
                input_file.seek(std::io::SeekFrom::Start(start as u64)).unwrap();
                input_file.read_exact(&mut chunk).unwrap();
    
                //Place the pixels onto the frame
                for (j, byte) in chunk.iter().enumerate() {
                    for bit_index in 0..8 {
                        let bit = (byte & (1 << bit_index)) != 0;
                        let color = if bit { Rgb([255, 255, 255]) } else { Rgb([0, 0, 0]) };
                        let pixel_x = ((j * 8 * encode.square_w) % encode.res.0 + bit_index * encode.square_w) as u32;
                        let pixel_y = (((j * 8 * encode.square_h) / encode.res.0 ) * encode.square_h) as u32;
                        for y in pixel_y..(pixel_y + encode.square_h as u32) {
                            for x in pixel_x..(pixel_x + encode.square_w as u32) {
                                frame.put_pixel(x, y, color);
                            }
                        }  
                    }
                }
    
                //Allocate the frame to the ffmpeg process
                {
                    let mut process = ffmpeg_process.lock().unwrap();
                    process.stdin.as_mut().unwrap().write_all(&frame.into_raw()).unwrap();
                }
    
                // Release used memory
                drop(chunk);
                drop(input_file);
    
            });
        output.to_string()
    }
}


pub fn decoder(file: std::path::PathBuf) {
    // Output file
    let output_file = Arc::new(Mutex::new(File::create("output.txt").unwrap()));

    // Get the number of frames in the video
    let num_frames_output = Command::new("ffprobe")
        .args(&[
            "-v", "error",
            "-select_streams", "v:0",
            "-count_frames",
            "-show_entries", "stream=nb_read_frames",
            "-print_format", "default=nokey=1:noprint_wrappers=1",
            file.to_str().unwrap(),
        ])
        .stdin(std::process::Stdio::piped())
        .output()
        .unwrap();

    let num_frames_str = String::from_utf8(num_frames_output.stdout).unwrap().trim().to_owned();
    let num_frames = num_frames_str.parse::<u32>().unwrap();

    // Set up FFmpeg command to output raw video data in grayscale format
    let mut ffmpeg = Command::new("ffmpeg");
    ffmpeg.arg("-y")
        .arg("-i")
        .arg(file)
        .arg("-f")
        .arg("image2pipe")
        .arg("-pix_fmt")
        .arg("gray")
        .arg("-vcodec")
        .arg("rawvideo")
        .arg("-")
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null());

    // Spawn FFmpeg process and capture stdout
    let mut child = ffmpeg.spawn().expect("Failed to spawn FFmpeg process");
    let stdout = Arc::new(Mutex::new(child.stdout.take().unwrap()));
    
    // Process each frame in parallel
    (0..num_frames).into_par_iter().for_each(|i| {
        // Read raw video data from stdout
        let mut bytes_read = 0;
        let mut frame_data = vec![0; 1920 * 1080 * 3];
        let mut stdout = stdout.lock().unwrap();
        {
            let stdout_ref = &mut *stdout;

            while bytes_read < frame_data.len() {
                match stdout_ref.read(&mut frame_data[bytes_read..]) {
                    Ok(0) => {
                        // End of stream reached before buffer was filled
                        break;
                    },
                    Ok(n) => {
                        bytes_read += n;
                    },
                    Err(ref e) if e.kind() == std::io::ErrorKind::Interrupted => {
                        // Read was interrupted, try again
                        continue;
                    },
                    Err(e) => {
                        // Error reading data
                        panic!("Error reading data from stdout: {:?}", e);
                    }
                }
            }
            
            // Pad the remaining space in the buffer with zeros
            for i in bytes_read..frame_data.len() {
                frame_data[i] = 0;
            }
        }

        println!("{}", i);

        // Convert raw video data to grayscale image
        let image = ImageBuffer::from_raw(1920, 1080, frame_data).unwrap();
        let gray_image = image::DynamicImage::ImageLuma8(image);

        // Divide image into 48x32 chunks and process each chunk
        let mut buffer = Vec::new();
        for chunk in get_chunks(gray_image, 4, 4) {
            // Get average brightness of chunk
            let brightness = image::GenericImageView::pixels(&chunk)
            .map(|p| {
                let r = (p.0 >> 16) & 0xFF;
                let g = (p.0 >> 8) & 0xFF;
                let b = p.0 & 0xFF;
                (r as f32 + g as f32 + b as f32) / (3.0 * 255.0)
            })
            .sum::<f32>() / (48.0 * 32.0);

            // Convert brightness to 0 or 1 based on a threshold
            let bit = if brightness > 128.0 { 1 } else { 0 };

            // Add bit to buffer
            buffer.push(bit);

            // If buffer is full, write bytes to output file
            if buffer.len() == 8 {
                let byte = buffer.iter().enumerate().fold(0, |acc, (i, bit)| acc | (bit << (7 - i)));
                let mut output_file_lock = output_file.lock().unwrap();
                let mut output_file = std::ops::DerefMut::deref_mut(&mut output_file_lock);
                output_file.write_all(&[byte]).unwrap();
                buffer.clear();
            }
        }
        println!("{} Complete!", i);
    });

    // Wait for FFmpeg process to exit
    child.wait().unwrap();
}

// Get the number of frames in the input file
fn num_frames(input_file: &str) -> usize {
    let output = Command::new("ffprobe")
        .args(&["-v", "error", "-select_streams", "v:0", "-count_frames", "-show_entries", "stream=nb_frames"])
        .arg(input_file)
        .output()
        .expect("Failed to execute ffprobe");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let re = regex::Regex::new(r"nb_frames=(\d+)").unwrap();
    let caps = re.captures(&stdout).unwrap();
    caps[1].parse().unwrap()
}

fn process_frame(frame: image::DynamicImage) -> Vec<u8> {
    let (width, height) = image::GenericImageView::dimensions(&frame);
    let mut result = Vec::new();

    let chunk_width = width / 48;
    let chunk_height = height / 32;

    for y in 0..32 {
        for x in 0..48 {
            let chunk = image::GenericImage::sub_image(&mut frame.to_luma8(), x * chunk_width, y * chunk_height, chunk_width, chunk_height)
                .to_image();
            let color = get_chunk_color(chunk);
            result.push(color);
        }
    }

    result
}

fn get_chunk_color(chunk: ImageBuffer<image::Luma<u8>, Vec<u8>>) -> u8 {
    let mut sum = 0;

    for pixel in chunk.pixels() {
        sum += pixel[0] as u32;
    }

    let average = sum / (chunk.width() * chunk.height()) as u32;

    if average >= 128 {
        1
    } else {
        0
    }
}

fn convert(bits: &[u8]) -> u8 {
    let mut result: u8 = 0;
    bits.iter().for_each(|&bit| {
        result <<= 1;
        result ^= bit;
    });
    result
}

fn get_chunks(mut image: image::DynamicImage, chunk_width: u32, chunk_height: u32) -> Vec<image::DynamicImage> {
    let (image_width, image_height) = image::GenericImageView::dimensions(&image);
    let mut chunks = Vec::new();
    for y in (0..image_height).step_by(chunk_height as usize) {
        for x in (0..image_width).step_by(chunk_width as usize) {
            let chunk = image.crop(x, y, chunk_width, chunk_height);
            chunks.push(chunk);
        }
    }
    chunks
}