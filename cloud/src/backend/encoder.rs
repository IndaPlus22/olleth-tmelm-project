use image::{ImageBuffer, Rgb};
use std::fs::File;
use std::io::{Read, Write, Seek};
use std::process::Command;

use crate::backend::file::FileInfo;

use std::sync::{Arc, Mutex};
use indicatif::{ProgressBar, ProgressStyle};
use rayon::prelude::*;



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
        let output = &format!("output/{}.mp4", FileInfo::name(&encode.file));
    
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


