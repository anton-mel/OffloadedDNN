use crate::camera::Camera;
use crate::server_facing::ServerFacing;

use opencv::highgui;
use std::time::Instant;

pub struct App {
    server: ServerFacing,
    camera: Camera,
}

impl App {
    pub fn new(server_address: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let server = ServerFacing::new(server_address)?;
        let camera = Camera::new("/dev/video0")?;
        Ok(App { server, camera })
    }

    pub fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.camera.start_streaming()?;
        
        opencv::highgui::named_window("MoveNet (CPSC 429)", opencv::highgui::WINDOW_AUTOSIZE)?;

        let mut frame_count = 0;
        let start_time = Instant::now();

        loop {
            let frame = self.camera.get_frame()?;
            self.server.send_image(frame)?;
            let (_inference_result, rgb_image) = self.server.receive_result()?;
            self.render(&rgb_image)?;

            frame_count += 1;
            if frame_count % 30 == 0 {
                let elapsed = start_time.elapsed();
                let fps = frame_count as f64 / elapsed.as_secs_f64();
                println!("FPS: {:.2}", fps);
            }

            if highgui::wait_key(1)? > 0 {
                break;
            }
        }

        self.camera.stop_streaming()?;
        Ok(())
    }

    fn render(&self, rgb_frame: &[u8]) -> Result<(), Box<dyn std::error::Error>> {
        // Decode the image (JPEG compressed)
        let img = match opencv::imgcodecs::imdecode(&opencv::core::Vector::from_slice(rgb_frame), opencv::imgcodecs::IMREAD_COLOR) {
            Ok(img) => img,
            Err(e) => {
                println!("Failed to decode image: {:?}", e);
                return Ok(());
            }
        };

        // Display the image
        opencv::highgui::imshow("MoveNet (CPSC 429)", &img)?;

        // Process GUI events
        opencv::highgui::wait_key(1)?;

        Ok(())
    }
}
