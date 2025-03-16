use std::io::Cursor;

use tray_icon::{
    TrayIcon, TrayIconBuilder, TrayIconEvent,
    menu::{Menu, MenuEvent, MenuItem},
};
use winit::{application::ApplicationHandler, event_loop::EventLoop};

#[derive(Debug)]
enum UserEvent {
    TrayIconEvent,
    MenuEvent,
}

struct Application {
    tray_icon: Option<TrayIcon>,
}

impl Application {
    fn new() -> Application {
        Application { tray_icon: None }
    }

    fn new_tray_icon() -> TrayIcon {
        let icon = load_icon();

        TrayIconBuilder::new()
            .with_menu(Box::new(Self::new_tray_menu()))
            .with_tooltip("AudioSwitch - Change Audio Endpoints")
            .with_icon(icon)
            .with_title("AudioSwitch")
            .build()
            .unwrap()
    }

    fn new_tray_menu() -> Menu {
        let menu = Menu::new();
        let exit_entry = MenuItem::new("Quit", true, None);
        if let Err(err) = menu.append(&exit_entry) {
            println!("{err:?}");
        }
        menu
    }
}

impl ApplicationHandler<UserEvent> for Application {
    fn resumed(&mut self, _event_loop: &winit::event_loop::ActiveEventLoop) {}

    fn window_event(
        &mut self,
        _event_loop: &winit::event_loop::ActiveEventLoop,
        _window_id: winit::window::WindowId,
        _event: winit::event::WindowEvent,
    ) {
    }

    fn new_events(
        &mut self,
        _event_loop: &winit::event_loop::ActiveEventLoop,
        cause: winit::event::StartCause,
    ) {
        if winit::event::StartCause::Init == cause {
            self.tray_icon = Some(Self::new_tray_icon());
        }
    }
    fn user_event(&mut self, event_loop: &winit::event_loop::ActiveEventLoop, event: UserEvent) {
        match event {
            UserEvent::MenuEvent => event_loop.exit(),
            _ => {}
        }
    }
}

pub fn create_tray() {
    let event_loop = EventLoop::<UserEvent>::with_user_event().build().unwrap();

    // set a tray event handler that forwards the event and wakes up the event loop
    let proxy = event_loop.create_proxy();
    TrayIconEvent::set_event_handler(Some(move |_event| {
        let _ = proxy.send_event(UserEvent::TrayIconEvent);
    }));
    let proxy = event_loop.create_proxy();
    MenuEvent::set_event_handler(Some(move |_event| {
        let _ = proxy.send_event(UserEvent::MenuEvent);
    }));

    let mut app = Application::new();

    let _menu_channel = MenuEvent::receiver();
    let _tray_channel = TrayIconEvent::receiver();

    if let Err(err) = event_loop.run_app(&mut app) {
        println!("Error: {:?}", err);
    }
}

fn load_icon() -> tray_icon::Icon {
    let (icon_rgba, icon_width, icon_height) = {
        let img_bytes = include_bytes!("../assets/app.png");
        let mut image = image::ImageReader::new(Cursor::new(img_bytes));
        image.set_format(image::ImageFormat::Png);
        let rgb_img = image.decode().expect("Failed to load image.").into_rgba8();
        let (width, height) = rgb_img.dimensions();
        let rgba = rgb_img.into_raw();
        (rgba, width, height)
    };
    tray_icon::Icon::from_rgba(icon_rgba, icon_width, icon_height).expect("Failed to open icon")
}
