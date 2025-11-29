use native_windows_gui as nwg;
use std::sync::{Arc, Mutex};
use std::cell::RefCell;
use crate::config::AppConfig;
use crate::drive_monitor::DriveMonitor;

pub struct TrayApp {
    window: nwg::MessageWindow,
    icon: nwg::Icon,
    tray: nwg::TrayNotification,
    tray_menu: nwg::Menu,
    menu_title: nwg::MenuItem,
    menu_sep1: nwg::MenuSeparator,
    menu_settings: nwg::MenuItem,
    menu_schedules: nwg::MenuItem,
    menu_about: nwg::MenuItem,
    menu_sep2: nwg::MenuSeparator,
    menu_exit: nwg::MenuItem,
    
    config: Arc<Mutex<AppConfig>>,
    drive_monitor: Arc<Mutex<DriveMonitor>>,
    
    handler: RefCell<Option<nwg::EventHandler>>,
}

impl TrayApp {
    pub fn build_ui(
        config: Arc<Mutex<AppConfig>>,
        drive_monitor: Arc<Mutex<DriveMonitor>>,
    ) -> Result<Arc<Self>, nwg::NwgError> {
        // Create window
        let mut window = Default::default();
        nwg::MessageWindow::builder().build(&mut window)?;
        
        // Create icon
        let mut icon = Default::default();
        nwg::Icon::builder()
            .source_bin(Some(include_bytes!("../assets/icon.ico")))
            .build(&mut icon)
            .unwrap_or_else(|_| {
                // Fallback to default icon if custom not found
                nwg::Icon::builder()
                    .build(&mut icon)
                    .expect("Failed to create icon");
            });
        
        // Create tray
        let mut tray = Default::default();
        nwg::TrayNotification::builder()
            .parent(&window)
            .icon(Some(&icon))
            .tip(Some("DriveGuard - Automatic Backup"))
            .build(&mut tray)?;
        
        // Create menu
        let mut tray_menu = Default::default();
        nwg::Menu::builder()
            .parent(&window)
            .popup(true)
            .build(&mut tray_menu)?;
        
        let mut menu_title = Default::default();
        nwg::MenuItem::builder()
            .text("DriveGuard v0.1.0")
            .parent(&tray_menu)
            .disabled(true)
            .build(&mut menu_title)?;
        
        let mut menu_sep1 = Default::default();
        nwg::MenuSeparator::builder()
            .parent(&tray_menu)
            .build(&mut menu_sep1)?;
        
        let mut menu_settings = Default::default();
        nwg::MenuItem::builder()
            .text("Settings")
            .parent(&tray_menu)
            .build(&mut menu_settings)?;
        
        let mut menu_schedules = Default::default();
        nwg::MenuItem::builder()
            .text("View Schedules")
            .parent(&tray_menu)
            .build(&mut menu_schedules)?;
        
        let mut menu_about = Default::default();
        nwg::MenuItem::builder()
            .text("About")
            .parent(&tray_menu)
            .build(&mut menu_about)?;
        
        let mut menu_sep2 = Default::default();
        nwg::MenuSeparator::builder()
            .parent(&tray_menu)
            .build(&mut menu_sep2)?;
        
        let mut menu_exit = Default::default();
        nwg::MenuItem::builder()
            .text("Exit")
            .parent(&tray_menu)
            .build(&mut menu_exit)?;
        
        let app = Arc::new(TrayApp {
            window,
            icon,
            tray,
            tray_menu,
            menu_title,
            menu_sep1,
            menu_settings,
            menu_schedules,
            menu_about,
            menu_sep2,
            menu_exit,
            config,
            drive_monitor,
            handler: RefCell::new(None),
        });
        
        // Setup event handlers
        let app_clone = app.clone();
        let handler = nwg::full_bind_event_handler(&app.window.handle, move |evt, _evt_data, handle| {
            use nwg::Event;
            
            if handle == app_clone.tray {
                match evt {
                    Event::OnContextMenu => {
                        let (x, y) = nwg::GlobalCursor::position();
                        app_clone.tray_menu.popup(x, y);
                    }
                    Event::OnMousePress(nwg::MousePressEvent::MousePressLeftUp) => {
                        let (x, y) = nwg::GlobalCursor::position();
                        app_clone.tray_menu.popup(x, y);
                    }
                    _ => {}
                }
            } else if handle == app_clone.menu_settings {
                if let Event::OnMenuItemSelected = evt {
                    app_clone.show_settings();
                }
            } else if handle == app_clone.menu_schedules {
                if let Event::OnMenuItemSelected = evt {
                    app_clone.show_schedules();
                }
            } else if handle == app_clone.menu_about {
                if let Event::OnMenuItemSelected = evt {
                    app_clone.show_about();
                }
            } else if handle == app_clone.menu_exit {
                if let Event::OnMenuItemSelected = evt {
                    nwg::stop_thread_dispatch();
                }
            }
        });
        
        *app.handler.borrow_mut() = Some(handler);
        
        Ok(app)
    }
    
    fn show_settings(&self) {
        if let Ok(cfg) = self.config.lock() {
            let msg = format!(
                "Current Settings:\n\n\
                Language: {}\n\
                Min Free Space: {} GB\n\
                Warn Before Delete: {}\n\
                Active Schedules: {}\n\n\
                Edit 'settings.toml' to change settings.",
                cfg.general.language,
                cfg.general.min_free_space_gb,
                cfg.general.warn_before_delete,
                cfg.schedules.len()
            );
            
            nwg::modal_info_message(&self.window, "Settings", &msg);
        }
    }
    
    fn show_schedules(&self) {
        if let Ok(cfg) = self.config.lock() {
            if cfg.schedules.is_empty() {
                nwg::modal_info_message(
                    &self.window,
                    "Schedules",
                    "No schedules configured yet.\n\nAdd a schedule in settings.toml to get started!"
                );
            } else {
                let mut msg = String::from("Configured Schedules:\n\n");
                for schedule in &cfg.schedules {
                    msg.push_str(&format!(
                        "• {} ({})\n  Interval: {} days\n  Trigger on connect: {}\n  Destination: {}\n\n",
                        schedule.name,
                        if schedule.enabled { "Enabled" } else { "Disabled" },
                        schedule.interval_days,
                        schedule.trigger_on_connect,
                        schedule.destination_path
                    ));
                }
                
                nwg::modal_info_message(&self.window, "Schedules", &msg);
            }
        }
    }
    
    fn show_about(&self) {
        nwg::modal_info_message(
            &self.window,
            "About DriveGuard",
            "DriveGuard v0.1.0 (Bare Bones)\n\n\
            Automatic USB Drive Backup Tool\n\n\
            Features:\n\
            • Drive detection by serial number\n\
            • Schedule-based backups\n\
            • Full file copy with structure preservation\n\n\
            Created with Rust 🦀"
        );
    }
}

impl Drop for TrayApp {
    fn drop(&mut self) {
        let handler = self.handler.borrow();
        if let Some(h) = handler.as_ref() {
            nwg::unbind_event_handler(h);
        }
    }
}