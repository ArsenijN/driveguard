use native_windows_gui as nwg;
use std::sync::{Arc, Mutex};
use std::cell::RefCell;
use std::thread;
use std::time::Duration;
use crate::config::BackupSchedule;
use crate::backup::BackupEngine;

pub struct CountdownWindow {
    window: nwg::Window,
    
    label_title: nwg::Label,
    label_countdown: nwg::Label,
    label_warning: nwg::Label,
    
    btn_start_now: nwg::Button,
    btn_hide: nwg::Button,
    btn_cancel: nwg::Button,
    
    timer: nwg::AnimationTimer,
    
    schedule: Arc<Mutex<BackupSchedule>>,
    seconds_remaining: Arc<Mutex<u64>>,
    cancelled: Arc<Mutex<bool>>,
    
    handler: RefCell<Option<nwg::EventHandler>>,
}

impl CountdownWindow {
    pub fn show(schedule: BackupSchedule, drive_letter: char) {
        log::info!("CountdownWindow::show called for drive {}", drive_letter);
        log::info!("Creating countdown window for drive {}", drive_letter);
        
        thread::spawn(move || {
            log::info!("Countdown window thread started for drive {}", drive_letter);
            
            if let Err(e) = nwg::init() {
                log::error!("Failed to init NWG in countdown thread: {:?}", e);
                return;
            }
            
            log::info!("NWG initialized in countdown thread");
            
            let seconds = schedule.countdown_minutes * 60;
            let schedule = Arc::new(Mutex::new(schedule));
            let seconds_remaining = Arc::new(Mutex::new(seconds));
            let cancelled = Arc::new(Mutex::new(false));
            
            let mut window = Default::default();
            if let Err(e) = nwg::Window::builder()
                .size((500, 250))
                .position((300, 300))
                .title("DriveGuard - Backup Starting")
                .flags(nwg::WindowFlags::WINDOW | nwg::WindowFlags::VISIBLE)
                .build(&mut window) {
                log::error!("Failed to build countdown window: {:?}", e);
                return;
            }
            
            log::info!("Countdown window created successfully");
            
            let mut label_title = Default::default();
            nwg::Label::builder()
                .text(&crate::localization::tf("backup_starting", &[&drive_letter.to_string()]))
                .parent(&window)
                .position((20, 20))
                .size((460, 30))
                .build(&mut label_title)
                .expect("Failed to build title label");
            
            let mut label_countdown = Default::default();
            nwg::Label::builder()
                .text(&format!("Starting in {}:{:02}", seconds / 60, seconds % 60))
                .parent(&window)
                .position((20, 60))
                .size((460, 40))
                .h_align(nwg::HTextAlign::Center)
                .build(&mut label_countdown)
                .expect("Failed to build countdown label");
            
            let mut label_warning = Default::default();
            nwg::Label::builder()
                .text(&crate::localization::t("do_not_disconnect"))
                .parent(&window)
                .position((20, 110))
                .size((460, 40))
                .build(&mut label_warning)
                .expect("Failed to build warning label");
            
            let mut btn_start_now = Default::default();
            nwg::Button::builder()
                .text(&crate::localization::t("button_start_now"))
                .parent(&window)
                .position((20, 170))
                .size((140, 40))
                .build(&mut btn_start_now)
                .expect("Failed to build start button");
            
            let mut btn_hide = Default::default();
            nwg::Button::builder()
                .text(&crate::localization::t("button_hide"))
                .parent(&window)
                .position((180, 170))
                .size((140, 40))
                .build(&mut btn_hide)
                .expect("Failed to build hide button");
            
            let mut btn_cancel = Default::default();
            nwg::Button::builder()
                .text(&crate::localization::t("button_cancel"))
                .parent(&window)
                .position((340, 170))
                .size((140, 40))
                .build(&mut btn_cancel)
                .expect("Failed to build cancel button");
            
            let mut timer = Default::default();
            nwg::AnimationTimer::builder()
                .parent(&window)
                .interval(Duration::from_secs(1))
                .build(&mut timer)
                .expect("Failed to build timer");
            
            let app = CountdownWindow {
                window,
                label_title,
                label_countdown,
                label_warning,
                btn_start_now,
                btn_hide,
                btn_cancel,
                timer,
                schedule,
                seconds_remaining,
                cancelled,
                handler: RefCell::new(None),
            };
            
            let app = Arc::new(app);
            
            // Setup event handlers
            let app_clone = app.clone();
            let handler = nwg::full_bind_event_handler(&app.window.handle, move |evt, _evt_data, handle| {
                use nwg::Event;
                
                if handle == app_clone.timer {
                    if let Event::OnTimerTick = evt {
                        app_clone.on_timer_tick();
                    }
                } else if handle == app_clone.btn_start_now {
                    if let Event::OnButtonClick = evt {
                        app_clone.start_backup_now();
                    }
                } else if handle == app_clone.btn_hide {
                    if let Event::OnButtonClick = evt {
                        app_clone.hide_window();
                    }
                } else if handle == app_clone.btn_cancel {
                    if let Event::OnButtonClick = evt {
                        app_clone.cancel_backup();
                    }
                } else if handle == app_clone.window {
                    if let Event::OnWindowClose = evt {
                        app_clone.cancel_backup();
                    }
                }
            });
            
            *app.handler.borrow_mut() = Some(handler);
            
            // Start the timer
            app.timer.start();
            
            nwg::dispatch_thread_events();
        });
    }
    
    fn on_timer_tick(&self) {
        let mut seconds = self.seconds_remaining.lock().unwrap();
        
        if *seconds > 0 {
            *seconds -= 1;
            let mins = *seconds / 60;
            let secs = *seconds % 60;
            self.label_countdown.set_text(&format!("Starting in {}:{:02}", mins, secs));
        } else {
            // Time's up, start backup
            self.timer.stop();
            self.start_backup_now();
        }
    }
    
    fn start_backup_now(&self) {
        log::info!("Starting backup now!");
        self.timer.stop();
        
        let schedule = self.schedule.lock().unwrap().clone();
        self.label_countdown.set_text("Backup in progress...");
        self.btn_start_now.set_enabled(false);
        self.btn_cancel.set_enabled(false);
        
        // Run backup
        let result = self.run_backup(&schedule);
        
        match result {
            Ok(backup_folder) => {
                log::info!("Backup completed successfully to: {}", backup_folder);
                nwg::modal_info_message(&self.window, "Backup Complete", 
                    &format!("Backup completed successfully!\n\nSaved to:\n{}", backup_folder));
            }
            Err(e) => {
                log::error!("Backup failed: {}", e);
                nwg::modal_error_message(&self.window, "Backup Failed", 
                    &format!("Backup failed:\n\n{}", e));
            }
        }
        
        nwg::stop_thread_dispatch();
    }
    
    fn run_backup(&self, schedule: &BackupSchedule) -> Result<String, String> {
        let mut engine = BackupEngine::new();
        
        // Load backup list
        let source_paths = schedule.load_backup_list();
        
        if source_paths.is_empty() {
            return Err("No source paths configured in backup list".to_string());
        }
        
        log::info!("Backing up {} paths to {}", source_paths.len(), schedule.destination_path);
        
        let backup_folder = engine.run_backup(&source_paths, &schedule.destination_path)?;
        
        // Save logs
        engine.save_logs(&backup_folder).ok();
        
        Ok(backup_folder)
    }
    
    fn hide_window(&self) {
        log::info!("Hiding countdown window");
        self.window.set_visible(false);
    }
    
    fn cancel_backup(&self) {
        log::info!("Backup cancelled by user");
        *self.cancelled.lock().unwrap() = true;
        nwg::stop_thread_dispatch();
    }
}

impl Drop for CountdownWindow {
    fn drop(&mut self) {
        let handler = self.handler.borrow();
        if let Some(h) = handler.as_ref() {
            nwg::unbind_event_handler(h);
        }
    }
}