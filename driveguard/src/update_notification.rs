use native_windows_gui as nwg;
use std::sync::{Arc, Mutex};
use std::cell::RefCell;
use std::thread;
use crate::update_checker::{UpdateInfo, UpdateChecker};
use crate::config::AppConfig;

pub struct UpdateNotificationWindow {
    window: nwg::Window,
    
    label_title: nwg::Label,
    label_version: nwg::Label,
    label_size: nwg::Label,
    label_info: nwg::Label,
    
    btn_update_now: nwg::Button,
    btn_ask_later: nwg::Button,
    btn_skip_version: nwg::Button,
    
    update_info: Arc<Mutex<UpdateInfo>>,
    config: Arc<Mutex<AppConfig>>,
    
    handler: RefCell<Option<nwg::EventHandler>>,
}

impl UpdateNotificationWindow {
    pub fn show(update_info: UpdateInfo, config: Arc<Mutex<AppConfig>>) {
        thread::spawn(move || {
            nwg::init().expect("Failed to init NWG");
            
            let update_info = Arc::new(Mutex::new(update_info));
            let info = update_info.lock().unwrap().clone();
            
            let mut window = Default::default();
            nwg::Window::builder()
                .size((500, 300))
                .position((300, 300))
                .title("DriveGuard Update Available")
                .flags(nwg::WindowFlags::WINDOW | nwg::WindowFlags::VISIBLE)
                .build(&mut window)
                .expect("Failed to build window");
            
            let mut label_title = Default::default();
            nwg::Label::builder()
                .text("🎉 DriveGuard Update Available!")
                .parent(&window)
                .position((20, 20))
                .size((460, 30))
                .build(&mut label_title)
                .expect("Failed to build title");
            
            let mut label_version = Default::default();
            nwg::Label::builder()
                .text(&format!("Version {} is now available (you have {})", 
                              info.version, crate::update_checker::get_current_version()))
                .parent(&window)
                .position((20, 60))
                .size((460, 25))
                .build(&mut label_version)
                .expect("Failed to build version label");
            
            let size_mb = info.size_bytes as f64 / 1_048_576.0;
            let mut label_size = Default::default();
            nwg::Label::builder()
                .text(&format!("Download size: {:.2} MB", size_mb))
                .parent(&window)
                .position((20, 90))
                .size((460, 25))
                .build(&mut label_size)
                .expect("Failed to build size label");
            
            let breaking_text = if info.breaking_changes {
                "\n⚠ This update contains breaking changes. Please review the changelog."
            } else {
                "\nThis is a compatible update and can be installed safely."
            };
            
            let mut label_info = Default::default();
            nwg::Label::builder()
                .text(&format!("Changes:\n- Bug fixes and improvements\n- Enhanced performance{}\n\nTo disable automatic updates, go to Settings > Updates", breaking_text))
                .parent(&window)
                .position((20, 120))
                .size((460, 100))
                .build(&mut label_info)
                .expect("Failed to build info label");
            
            let mut btn_update_now = Default::default();
            nwg::Button::builder()
                .text("Update Now")
                .parent(&window)
                .position((20, 230))
                .size((140, 40))
                .build(&mut btn_update_now)
                .expect("Failed to build update button");
            
            let mut btn_ask_later = Default::default();
            nwg::Button::builder()
                .text("Ask Me Later")
                .parent(&window)
                .position((180, 230))
                .size((140, 40))
                .build(&mut btn_ask_later)
                .expect("Failed to build later button");
            
            let mut btn_skip_version = Default::default();
            nwg::Button::builder()
                .text("Skip This Version")
                .parent(&window)
                .position((340, 230))
                .size((140, 40))
                .build(&mut btn_skip_version)
                .expect("Failed to build skip button");
            
            let app = UpdateNotificationWindow {
                window,
                label_title,
                label_version,
                label_size,
                label_info,
                btn_update_now,
                btn_ask_later,
                btn_skip_version,
                update_info,
                config,
                handler: RefCell::new(None),
            };
            
            let app = Arc::new(app);
            
            // Setup event handlers
            let app_clone = app.clone();
            let handler = nwg::full_bind_event_handler(&app.window.handle, move |evt, _evt_data, handle| {
                use nwg::Event;
                
                if handle == app_clone.btn_update_now {
                    if let Event::OnButtonClick = evt {
                        app_clone.start_update();
                    }
                } else if handle == app_clone.btn_ask_later {
                    if let Event::OnButtonClick = evt {
                        log::info!("Update postponed by user");
                        nwg::stop_thread_dispatch();
                    }
                } else if handle == app_clone.btn_skip_version {
                    if let Event::OnButtonClick = evt {
                        app_clone.skip_version();
                    }
                } else if handle == app_clone.window {
                    if let Event::OnWindowClose = evt {
                        nwg::stop_thread_dispatch();
                    }
                }
            });
            
            *app.handler.borrow_mut() = Some(handler);
            
            nwg::dispatch_thread_events();
        });
    }
    
    fn start_update(&self) {
        log::info!("User chose to update now");
        
        self.label_title.set_text("Downloading update...");
        self.btn_update_now.set_enabled(false);
        self.btn_ask_later.set_enabled(false);
        self.btn_skip_version.set_enabled(false);
        
        let info = self.update_info.lock().unwrap().clone();
        let config = self.config.lock().unwrap();
        let checker = UpdateChecker::new(&config);
        drop(config);
        
        // Download update
        match checker.download_update(&info) {
            Ok(path) => {
                log::info!("Download complete: {}", path);
                self.label_title.set_text("Download complete! Applying update...");
                
                // Apply update (this will exit DriveGuard)
                if let Err(e) = checker.apply_update(&info.version) {
                    log::error!("Failed to apply update: {}", e);
                    nwg::modal_error_message(&self.window, "Update Failed", 
                        &format!("Failed to apply update:\n\n{}", e));
                } else {
                    // This shouldn't be reached as apply_update exits the app
                    nwg::stop_thread_dispatch();
                }
            }
            Err(e) => {
                log::error!("Download failed: {}", e);
                nwg::modal_error_message(&self.window, "Download Failed", 
                    &format!("Failed to download update:\n\n{}", e));
                
                self.label_title.set_text("Update Available");
                self.btn_update_now.set_enabled(true);
                self.btn_ask_later.set_enabled(true);
                self.btn_skip_version.set_enabled(true);
            }
        }
    }
    
    fn skip_version(&self) {
        let info = self.update_info.lock().unwrap();
        log::info!("User chose to skip version {}", info.version);
        
        // Add version to skipped list
        if let Ok(mut config) = self.config.lock() {
            if let Some(ref mut update_settings) = config.general.update_settings {
                if !update_settings.skipped_versions.contains(&info.version) {
                    update_settings.skipped_versions.push(info.version.clone());
                    config.save();
                }
            }
        }
        
        nwg::stop_thread_dispatch();
    }
}

impl Drop for UpdateNotificationWindow {
    fn drop(&mut self) {
        let handler = self.handler.borrow();
        if let Some(h) = handler.as_ref() {
            nwg::unbind_event_handler(h);
        }
    }
}