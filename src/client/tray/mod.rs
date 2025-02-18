use std::path::{Path, PathBuf};

use tao::{
    event::Event,
    event_loop::{ControlFlow, EventLoop, EventLoopBuilder},
};
use tray_icon::{
    TrayIconBuilder, TrayIconEvent,
    menu::{AboutMetadata, IsMenuItem, Menu, MenuEvent, MenuId, MenuItem, PredefinedMenuItem},
};

pub enum TrayEvent {
    TrayIconEvent(tray_icon::TrayIconEvent),
    MenuEvent(tray_icon::menu::MenuEvent),
}

pub struct TrayMenu {
    menu: Menu,
    event_loop: EventLoop<TrayEvent>,
    quit_id: MenuId,
    icon_path: PathBuf,
}

impl TrayMenu {
    pub fn new(icon_path: PathBuf) -> anyhow::Result<Self> {
        let event_loop = EventLoopBuilder::<TrayEvent>::with_user_event().build();

        // set a tray event handler that forwards the event and wakes up the event loop
        let proxy = event_loop.create_proxy();
        TrayIconEvent::set_event_handler(Some(move |event| {
            proxy.send_event(TrayEvent::TrayIconEvent(event));
        }));

        // set a menu event handler that forwards the event and wakes up the event loop
        let proxy = event_loop.create_proxy();
        MenuEvent::set_event_handler(Some(move |event| {
            proxy.send_event(TrayEvent::MenuEvent(event));
        }));

        let menu = Menu::new();

        let quit_i = MenuItem::new("Quit", true, None);

        menu.append_items(&[
            &PredefinedMenuItem::about(
                None,
                Some(AboutMetadata {
                    name: Some("tao".to_string()),
                    copyright: Some("Copyright tao".to_string()),
                    ..Default::default()
                }),
            ),
            &PredefinedMenuItem::separator(),
            &quit_i,
        ])?;

        Ok(Self {
            menu,
            event_loop,
            quit_id: quit_i.into_id(),
            icon_path,
        })
    }

    pub fn run(self) {
        let mut tray_icon = None;

        let menu_channel = MenuEvent::receiver();
        let tray_channel = TrayIconEvent::receiver();

        self.event_loop.run(move |event, _, control_flow| {
            *control_flow = ControlFlow::Wait;

            match event {
                Event::NewEvents(tao::event::StartCause::Init) => {
                    let icon = Self::load_icon(std::path::Path::new(&self.icon_path));

                    // We create the icon once the event loop is actually running
                    // to prevent issues like https://github.com/tauri-apps/tray-icon/issues/90
                    tray_icon = Some(
                        TrayIconBuilder::new()
                            .with_menu(Box::new(self.menu.clone()))
                            .with_tooltip("tao - awesome windowing lib")
                            .with_icon(icon)
                            .build()
                            .unwrap(),
                    );

                    // We have to request a redraw here to have the icon actually show up.
                    // Tao only exposes a redraw method on the Window so we use core-foundation directly.
                    #[cfg(target_os = "macos")]
                    unsafe {
                        use objc2_core_foundation::{CFRunLoopGetMain, CFRunLoopWakeUp};

                        let rl = CFRunLoopGetMain().unwrap();
                        CFRunLoopWakeUp(&rl);
                    }
                }

                Event::UserEvent(TrayEvent::TrayIconEvent(event)) => match event {
                    TrayIconEvent::Click {
                        id,
                        position,
                        rect,
                        button,
                        button_state,
                    } => {
                        println!("click");
                    }
                    TrayIconEvent::DoubleClick {
                        id,
                        position,
                        rect,
                        button,
                    } => {
                        println!("double click");
                    }
                    _ => {}
                },

                Event::UserEvent(TrayEvent::MenuEvent(event)) => {
                    println!("{event:?}");

                    if event.id == self.quit_id {
                        tray_icon.take();
                        *control_flow = ControlFlow::Exit;
                    }
                }

                _ => {}
            }
        })
    }

    fn load_icon(path: &std::path::Path) -> tray_icon::Icon {
        let (icon_rgba, icon_width, icon_height) = {
            let image = image::open(path)
                .expect("Failed to open icon path")
                .into_rgba8();
            let (width, height) = image.dimensions();
            let rgba = image.into_raw();
            (rgba, width, height)
        };
        tray_icon::Icon::from_rgba(icon_rgba, icon_width, icon_height).expect("Failed to open icon")
    }
}
