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
    dpi::{LogicalPosition, PhysicalPosition},
    platform::windows::WindowBorrowExtWindows,
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
struct App {
    mode: Mode,
    request_redraw: bool,
    wait_cancelled: bool,
    close_requested: bool,
    window: Option<Window>,
    mouse_positon: PhysicalPosition<f64>,
    drag_start: Option<PhysicalPosition<f64>>,
    //position: PhysicalPosition::<f64>,
}

impl App {
    fn select_range(&mut self, start: PhysicalPosition<f64>, end: PhysicalPosition<f64>) {
        capture_and_save_screenshot(Some(start), Some(end));
    }

    fn draw_selection(&self, window: &Window, start: PhysicalPosition<f64>, end: PhysicalPosition<f64>) {
        // Implement drawing selection rectangle on the screen
        // This is a placeholder for the actual drawing logic
        let (x, y, width, height) = (
            start.x.min(end.x),
            start.y.min(end.y),
            (start.x - end.x).abs(),
            (start.y - end.y).abs(),
        );
        println!("Drawing selection: start_x={} start_y={} width={} height={}", x, y, width, height);
    }
}

impl ApplicationHandler for App {
    fn new_events(&mut self, _event_loop: &ActiveEventLoop, cause: StartCause) {
        //info!("new_events: {cause:?}");

        self.wait_cancelled = match cause {
            StartCause::WaitCancelled { .. } => self.mode == Mode::WaitUntil,
            _ => false,
        }
    }

    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window_attributes = Window::default_attributes()
            .with_title("Drag & Drop capture screenshot",)
            .with_transparent(true)
            .with_fullscreen(Some(winit::window::Fullscreen::Borderless(None)))
            .with_decorations(false);
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
            WindowEvent::MouseInput { state, button, .. }=> {
                // TODO: implement drag and drop range selection
                match state {
                    ElementState::Pressed => {
                        if button == MouseButton::Left {
                            // TODO: get mouse position
                            self.drag_start = Some(self.mouse_positon);
                        }
                    },
                    ElementState::Released => {
                        if button == MouseButton::Left {
                            if let Some(start) = self.drag_start {
                                // When the mouse is released, the selection
                                let end = self.mouse_positon;
                                self.select_range(start, end);
                                self.drag_start = None;
                            }
                        }
                    }
                }
            },
            WindowEvent::CursorMoved { position, .. } => {
                self.mouse_positon = position;
                if let Some(start) = self.drag_start {
                    self.draw_selection(self.window.as_ref().unwrap(), start, self.mouse_positon);
                }
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
    
    let mut app = App::default();
    event_loop.run_app(&mut app);
}

fn capture_and_save_screenshot(start_pos: Option<PhysicalPosition<f64>>, end_pos: Option<PhysicalPosition<f64>>) {
    if let (Some(start), Some(end)) = (start_pos, end_pos) {
        let x_start = start.x.min(end.x) as usize;
        let y_start = start.y.min(end.y) as usize;
        let x_max = start.x.max(end.x) as usize;
        let y_max = start.y.max(end.y) as usize;

        let width = (x_max - x_start) as usize;
        let height = (y_max - x_start) as usize;

        // Setup Display and Capturer
        let display = Display::primary().expect("Couldn't find primary display.");
        let mut capturer = Capturer::new(display).expect("Couldn't begin capture.");

        // Capture loop
        let screen_width = &capturer.width();
        let screen_height = &capturer.height();

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
                let i = y * stride + x * 4;  // One dimensional index
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
                let src_y = y_start + y;
                let src_x = x_start + x;
                if src_y < *screen_height && src_x < *screen_width {
                    let src_index = (src_y as usize * stride + src_x as usize * 4) as usize;
                    let dst_index = (y * width + x) * 4;
                    cropped_buffer[dst_index..dst_index + 4].copy_from_slice(&buffer[src_index..src_index + 4]);
                }
            }
        }
        
        let mut image = ImageBuffer::<Rgba<u8>, _>::from_raw(width as u32, height as u32, cropped_buffer).expect("Couldn't create image buffer.");

        // Test paint. Fill red color at upper left
        for (x, y, pixel) in image.enumerate_pixels_mut() {
            if x < 100 && y < 20 {
                // Red fill at upper left 
                *pixel = Rgba([255, 0, 0, 255]);
            }
        }

        image.save("screenshot.png").expect("Couldn't save image.");
        println!("Screen shot saved as screenshot.png");
    }
}
