use scrap::{Capturer, Display};
use std::io::ErrorKind::WouldBlock;
use std::thread;
use std::time::Duration;
use image::{ImageBuffer, Rgba};

fn main() {
    // Setup Display and Capturer
    let display = Display::primary().expect("Couldn't find primary display.");
    let mut capturer = Capturer::new(display).expect("Couldn't begin capture.");

    // Capture loop
    loop {
        let width = &capturer.width();
        let height = &capturer.height();

        // Capture frame
        let frame = match capturer.frame() {
            Ok(buffer) => buffer,
            Err(error) => {
                if error.kind() == WouldBlock {
                    // Wait for the next frame
                    thread::sleep(Duration::from_millis(100));
                    continue;
                } else {
                    panic!("Error: {}", error);
                }
            }
        };

        let stride = frame.len() / *height; // Bytes per a row
        let mut buffer = vec![0; frame.len()];
        //let buffer = frame.to_vec();

        // Change BGRA to RGBA
        for y in 0..*height {
            for x in 0..*width {
                let i = y * stride + x * 4;
                buffer[i] = frame[i + 2]; // R
                buffer[i + 1] = frame[i + 1]; // G
                buffer[i + 2] = frame[i]; // B
                buffer[i + 3] = frame[i + 3]; // A
            }
        }

        let mut image = ImageBuffer::<Rgba<u8>, _>::from_raw(*width as u32, *height as u32, buffer).expect("Couldn't create image buffer.");

        for (x, y, pixel) in image.enumerate_pixels_mut() {
            if x < 100 && y < 20 {
                // Red fill at upper left 
                *pixel = Rgba([255, 0, 0, 255]);
            }
        }

        image.save("screenshot.png").expect("Couldn't save image.");
        break;
    }
}
