use scrap::{Capturer, Display};
use std::io::ErrorKind::WouldBlock;
use std::thread;
use std::time::Duration;
use std::time;


use image::{ImageBuffer, Rgba};
use winit::{
    application::ApplicationHandler,
    event::{KeyEvent, Event, WindowEvent, MouseButton, ElementState, StartCause},
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    keyboard::{Key, NamedKey},
    window::{Window, WindowId},
    dpi::LogicalPosition,
};


const WAIT_TIME: time::Duration = time::Duration::from_millis(100);
const POLL_SLEEP_TIME: time::Duration = time::Duration::from_millis(100);


#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
enum Mode {
    #[default]
    Wait,
    WaitUntil,
    Poll,
}

#[derive(Default)]
struct ApplicationControlFlow {
    mode: Mode,
    request_redraw: bool,
    wait_cancelled: bool,
    close_requested: bool,
    window: Option<Window>,
}

impl ApplicationHandler for ApplicationControlFlow {
    fn new_events(&mut self, _event_loop: &ActiveEventLoop, cause: StartCause) {
        //info!("new_events: {cause:?}");

        self.wait_cancelled = match cause {
            StartCause::WaitCancelled { .. } => self.mode == Mode::WaitUntil,
            _ => false,
        }
    }

    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window_attributes = Window::default_attributes().with_title(
            "Press 1, 2, 3 to change control flow mode. Press R to toggle redraw requests.",
        );
        self.window = Some(event_loop.create_window(window_attributes).unwrap());
    }

    fn window_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        //info!("{event:?}");

        match event {
            WindowEvent::CloseRequested => {
                self.close_requested = true;
            },
            WindowEvent::KeyboardInput {
                event: KeyEvent { logical_key: key, state: ElementState::Pressed, .. },
                ..
            } => match key.as_ref() {
                // WARNING: Consider using `key_without_modifiers()` if available on your platform.
                // See the `key_binding` example
                Key::Character("1") => {
                    self.mode = Mode::Wait;
                    //warn!("mode: {:?}", self.mode);
                },
                Key::Character("2") => {
                    self.mode = Mode::WaitUntil;
                    //warn!("mode: {:?}", self.mode);
                },
                Key::Character("3") => {
                    self.mode = Mode::Poll;
                    //warn!("mode: {:?}", self.mode);
                },
                Key::Character("r") => {
                    self.request_redraw = !self.request_redraw;
                    //warn!("request_redraw: {}", self.request_redraw);
                },
                Key::Named(NamedKey::Escape) => {
                    self.close_requested = true;
                },
                _ => (),
            },
            WindowEvent::RedrawRequested => {},
            _ => (),
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        if self.request_redraw && !self.wait_cancelled && !self.close_requested {
            self.window.as_ref().unwrap().request_redraw();
        }

        match self.mode {
            Mode::Wait => event_loop.set_control_flow(ControlFlow::Wait),
            Mode::WaitUntil => {
                if !self.wait_cancelled {
                    event_loop
                        .set_control_flow(ControlFlow::WaitUntil(time::Instant::now() + WAIT_TIME));
                }
            },
            Mode::Poll => {
                thread::sleep(POLL_SLEEP_TIME);
                event_loop.set_control_flow(ControlFlow::Poll);
            },
        };

        if self.close_requested {
            event_loop.exit();
        }
    }
}

fn main() {
    let event_loop = EventLoop::new().unwrap();
    

    let mut start_pos: Option<LogicalPosition<f64>> = None;
    let mut end_pos: Option<LogicalPosition<f64>> = None;
    let mut selecting = false;

    start_pos = Some(LogicalPosition::new(0.0, 0.0));
    end_pos = Some(LogicalPosition::new(1000.0, 1000.0));

    let mut app = ApplicationControlFlow::default();
    event_loop.run_app(&mut app);

    //event_loop.run(move |event, control_flow| {
    //    *control_flow = ControlFlow::Wait;

    //    match event {
    //        Event::WindowEvent { event, .. } => match event {
    //            WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
    //            WindowEvent::MouseInput { button, state, .. } => {
    //                if button == MouseButton::Left {
    //                    if state == ElementState::Pressed {
    //                        //let position = window.cursor_position().unwrap();
    //                        //start_pos = Some(position);
    //                        selecting = true;
    //                    } else if state == ElementState::Released {
    //                        //let position = window.cursor_position().unwrap();
    //                        //end_pos = Some(position);
    //                        selecting = false;
    //                        capture_and_save_screenshot(start_pos, end_pos);
    //                        *control_flow = ControlFlow::Exit;
    //                    }
    //                }
    //            }
    //            _ => (),
    //        },
    //        _ => (),
    //    }
    //});

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
