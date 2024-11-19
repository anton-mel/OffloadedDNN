use tflitec::interpreter::{Interpreter, Options};
use std::sync::mpsc::{channel, Sender, Receiver};
use std::net::{TcpListener, TcpStream};
use serde::{Serialize, Deserialize};
use opencv::core::{flip, Vec3b};
use std::io::{Read, Write};
use tflitec::model::Model;
use opencv::prelude::*;
use std::thread;

mod utils;
use utils::*;

#[derive(Serialize, Deserialize)]
struct InferenceResult {
    keypoints: Vec<f32>, // [1, 17, 3]
}

struct ModelInterpreter {
    interpreter: Interpreter<'static>,
}

impl ModelInterpreter {
    fn new(model_path: &str, options: Options) -> Result<Self, Box<dyn std::error::Error>> {
        let model = Model::new(model_path)?;
        let interpreter = unsafe {
            std::mem::transmute::<Interpreter<'_>, Interpreter<'static>>(
                Interpreter::new(&model, Some(options))?
            )
        };
        interpreter.allocate_tensors()?;
        Ok(Self { interpreter })
    }
}

fn handle_client(stream: TcpStream, model_path: String, options: Options) {
    println!("New client connected");
    let model_interpreter = ModelInterpreter::new(&model_path, options).expect("Failed to create ModelInterpreter");

    let (frame_sender, frame_receiver) = channel();
    let (result_sender, result_receiver) = channel();

    let stream_clone = stream.try_clone().unwrap();
    let receive_thread = thread::spawn(move || {
        receive_frames(stream, frame_sender);
    });

    let process_thread = thread::spawn(move || {
        process_frames(frame_receiver, result_sender, model_interpreter);
    });

    let send_thread = thread::spawn(move || {
        send_results(stream_clone, result_receiver);
    });

    receive_thread.join().unwrap();
    process_thread.join().unwrap();
    send_thread.join().unwrap();
    println!("Client disconnected");
}

fn receive_frames(mut stream: TcpStream, frame_sender: Sender<Vec<u8>>) {
    println!("Receive frames thread started");
    stream.set_read_timeout(Some(std::time::Duration::from_secs(5))).unwrap();
    loop {
        let mut len_buf = [0u8; 4];
        match stream.read_exact(&mut len_buf) {
            Ok(_) => {
                let len = u32::from_be_bytes(len_buf) as usize;
                
                let mut img_buf = vec![0u8; len];
                match stream.read_exact(&mut img_buf) {
                    Ok(_) => {
                        if frame_sender.send(img_buf).is_err() {
                            println!("Error sending frame to processing thread");
                            break;
                        }
                    },
                    Err(e) => {
                        println!("Error reading frame data: {:?}", e);
                        break;
                    }
                }
            },
            Err(e) => {
                if e.kind() == std::io::ErrorKind::WouldBlock {
                    println!("Read timeout, continuing...");
                    continue;
                } else {
                    println!("Client disconnected: {:?}", e);
                    break;
                }
            }
        }
    }
    println!("Receive frames thread ended");
}

fn process_frames(frame_receiver: Receiver<Vec<u8>>, result_sender: Sender<(InferenceResult, Vec<u8>)>, model_interpreter: ModelInterpreter) {
    while let Ok(img_buf) = frame_receiver.recv() {
        let rgb_frame = yuyv422_to_rgb(&img_buf);
        let original_mat = unsafe {
            Mat::new_rows_cols_with_data(
                1080,
                1920,
                opencv::core::CV_8UC3,
                rgb_frame.as_ptr() as *mut _,
                opencv::core::Mat_AUTO_STEP
            ).unwrap()
        };

        let mut flipped = Mat::default();
        flip(&original_mat, &mut flipped, 1).unwrap();
        let resized_img = resize_with_padding(&flipped, [192, 192]);
        let vec_2d: Vec<Vec<Vec3b>> = resized_img.to_vec_2d().unwrap();
        let vec_1d: Vec<u8> = vec_2d.iter().flat_map(|v| v.iter().flat_map(|w| w.as_slice())).cloned().collect();

        model_interpreter.interpreter.copy(&vec_1d[..], 0).unwrap();
        model_interpreter.interpreter.invoke().unwrap();
        let output_tensor = model_interpreter.interpreter.output(0).unwrap();
        let keypoints = output_tensor.data::<f32>().to_vec();

        let mut output_image = original_mat.clone();
        draw_keypoints(&mut output_image, &keypoints, 0.25);
        draw_connections(&mut output_image, &keypoints, 0.25); // Correct call to draw connections

        let mut img_buf = opencv::types::VectorOfu8::new();
        opencv::imgcodecs::imencode(".jpg", &output_image, &mut img_buf, &opencv::core::Vector::new()).unwrap();
        let img_bytes = img_buf.to_vec();

        let result = InferenceResult { keypoints };
        result_sender.send((result, img_bytes)).unwrap();
    }
}

fn send_results(mut stream: TcpStream, result_receiver: Receiver<(InferenceResult, Vec<u8>)>) {
    println!("Send frames thread started");
    while let Ok((result, img_bytes)) = result_receiver.recv() {
        
        let serialized = bincode::serialize(&result).unwrap();
        let result_len = (serialized.len() as u32).to_be_bytes();
        if stream.write_all(&result_len).is_err() || stream.write_all(&serialized).is_err() {
            println!("Error sending result data to client");
            break;
        }

        let img_len = (img_bytes.len() as u32).to_be_bytes();
        if stream.write_all(&img_len).is_err() || stream.write_all(&img_bytes).is_err() {
            println!("Error sending image data to client");
            break;
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let options = Options::default();
    let path = "resource/lite-model_movenet_singlepose_lightning_tflite_int8_4.tflite".to_string();

    let listener = TcpListener::bind("10.66.83.44:7878")?;
    println!("Server listening on 10.66.83.44:7878");

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let path_clone = path.clone();
                let options_clone = options.clone();
                thread::spawn(move || {
                    if let Err(e) = std::panic::catch_unwind(|| {
                        handle_client(stream, path_clone, options_clone);
                    }) {
                        eprintln!("Client thread panicked: {:?}", e);
                    }
                });
            }
            Err(e) => eprintln!("Connection failed: {:?}", e),
        }
    }

    Ok(())
}
