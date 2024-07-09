use scrap::{Capturer, Display};
use std::io::ErrorKind::WouldBlock;
use std::thread;
use std::time::Duration;
use image::{ImageBuffer, Rgba};
use winit::{
    event::{Event, WindowEvent, MouseButton, ElementState},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
    dpi::LogicalPosition,
};

fn main() {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("Select Area")
        .build(&event_loop)
        .unwrap();

    let mut start_pos: Option<LogicalPosition<f64>> = None;
    let mut end_pos: Option<LogicalPosition<f64>> = None;
    let mut selecting = false;

    start_pos = Some(LogicalPosition::new(0.0, 0.0));
    end_pos = Some(LogicalPosition::new(1000.0, 1000.0));

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                WindowEvent::MouseInput { button, state, .. } => {
                    if button == MouseButton::Left {
                        if state == ElementState::Pressed {
                            //let position = window.cursor_position().unwrap();
                            //start_pos = Some(position);
                            selecting = true;
                        } else if state == ElementState::Released {
                            //let position = window.cursor_position().unwrap();
                            //end_pos = Some(position);
                            selecting = false;
                            capture_and_save_screenshot(start_pos, end_pos);
                            *control_flow = ControlFlow::Exit;
                        }
                    }
                }
                _ => (),
            },
            _ => (),
        }
    });

}

fn capture_and_save_screenshot(start_pos: Option<LogicalPosition<f64>>, end_pos: Option<LogicalPosition<f64>>) {
    if let (Some(start), Some(end)) = (start_pos, end_pos) {
        let x_min = start.x.min(end.x) as i32;
        let y_min = start.y.min(end.y) as i32;
        let x_max = start.x.max(end.x) as i32;
        let y_max = start.y.max(end.y) as i32;

        let width = (x_max - x_min) as usize;
        let height = (y_max - y_min) as usize;

        // Setup Display and Capturer
        let display = Display::primary().expect("Couldn't find primary display.");
        let mut capturer = Capturer::new(display).expect("Couldn't begin capture.");

        //let width = &capturer.width();
        //let height = &capturer.height();

        // Capture loop
        let screen_width = &capturer.width();
        let screen_height = &capturer.height();

        // TODO: fix bug
        let frame = loop {
            match capturer.frame() {
                Ok(buffer) => break buffer,
                Err(error) => {
                    if error.kind() == WouldBlock {
                        thread::sleep(Duration::from_millis(100));
                        continue;
                    } else {
                        panic!("Error: {}", error);
                    }
                }
            }
        };

        let stride = frame.len() / screen_height; // bytes per line
        let mut buffer = vec![0; frame.len()];

        // Change BGRA to RGBA
        for y in 0..*screen_height {
            for x in 0..*screen_width {
                let i = y * stride + x * 4;
                buffer[i] = frame[i + 2]; // R
                buffer[i + 1] = frame[i + 1]; // G
                buffer[i + 2] = frame[i]; // B
                buffer[i + 3] = frame[i + 3]; // A
            }
        }

        // Crop the image
        let mut cropped_buffer = vec![0; width * height * 4];
        for y in 0..height {
            for x in 0..width {
                let src_index = ((y_min + y as i32) as usize * stride + (x_min + x as i32) as usize * 4) as usize;
                let dst_index = (y * width + x) * 4;
                cropped_buffer[dst_index..dst_index + 4].copy_from_slice(&buffer[src_index..src_index + 4]);
            }
        }

        let mut image = ImageBuffer::<Rgba<u8>, _>::from_raw(width as u32, height as u32, buffer).expect("Couldn't create image buffer.");

        for (x, y, pixel) in image.enumerate_pixels_mut() {
            if x < 100 && y < 20 {
                // Red fill at upper left 
                *pixel = Rgba([255, 0, 0, 255]);
            }
        }

        image.save("screenshot.png").expect("Couldn't save image.");
    }
}
