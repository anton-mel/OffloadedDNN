use std::net::TcpStream;
use std::io::{Read, Write};
use serde::{Serialize, Deserialize};
use bincode;

#[derive(Serialize, Deserialize)]
pub struct InferenceResult {
    pub keypoints: Vec<f32>, // [1, 17, 3]
}

pub struct ServerFacing {
    stream: TcpStream,
}

impl ServerFacing {
    pub fn new(address: &str) -> std::io::Result<Self> {
        let stream = TcpStream::connect(address)?;
        Ok(ServerFacing { stream })
    }

    pub fn send_image(&mut self, image_bytes: &[u8]) -> std::io::Result<()> {
        let len = (image_bytes.len() as u32).to_be_bytes();
        self.stream.write_all(&len)?;
        self.stream.write_all(image_bytes)?;
        Ok(())
    }

    pub fn receive_result(&mut self) -> std::io::Result<(InferenceResult, Vec<u8>)> {
        // Receive keypoints data
        let mut len_buf = [0u8; 4];
        self.stream.read_exact(&mut len_buf)?;
        let len = u32::from_be_bytes(len_buf) as usize;
        let mut data_buf = vec![0u8; len];
        self.stream.read_exact(&mut data_buf)?;
        let result: InferenceResult = bincode::deserialize(&data_buf).map_err(|e| {
            std::io::Error::new(std::io::ErrorKind::InvalidData, e)
        })?;
    
        // Receive image data
        self.stream.read_exact(&mut len_buf)?;
        let img_len = u32::from_be_bytes(len_buf) as usize;
        let mut img_buf = vec![0u8; img_len];
        self.stream.read_exact(&mut img_buf)?;
    
        Ok((result, img_buf))
    }
}
