// DriveGuard - Automatic USB Drive Backup Tool
// Version 0.1.0 - Bare Bones

// Comment out this line during development to see console logs:
// #![windows_subsystem = "windows"]

mod config;
mod drive_monitor;
mod backup;
mod ui;
mod localization;
mod countdown_window;
mod update_checker;
mod update_notification;
mod version;

use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use native_windows_gui as nwg;
use crate::config::AppConfig;
use crate::drive_monitor::DriveMonitor;
use crate::ui::TrayApp;

fn main() {
    // Initialize logging to console
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Info)
        .init();
    
    log::info!("DriveGuard v0.1.0 starting...");
    
    // Initialize NWG
    nwg::init().expect("Failed to init Native Windows GUI");
    
    // Load or create default configuration
    let config = Arc::new(Mutex::new(AppConfig::load_or_create()));
    
    // Set language from config
    if let Ok(cfg) = config.lock() {
        crate::localization::set_locale(&cfg.general.language);
        log::info!("Language set to: {}", cfg.general.language);
    }
    
    // Initialize drive monitor
    let drive_monitor = Arc::new(Mutex::new(DriveMonitor::new()));
    
    // Create and build the tray application
    let app = TrayApp::build_ui(config.clone(), drive_monitor.clone())
        .expect("Failed to build UI");
    
    // Check all drives on startup
    log::info!("Checking all connected drives on startup...");
    if let Ok(mut monitor) = drive_monitor.lock() {
        if let Ok(cfg) = config.lock() {
            monitor.check_all_drives_on_startup(&cfg);
        }
    }
    
    // Start drive monitoring thread
    let config_clone = config.clone();
    let drive_monitor_clone = drive_monitor.clone();
    thread::spawn(move || {
        loop {
            // Check for drive connections/disconnections
            if let Ok(mut monitor) = drive_monitor_clone.lock() {
                if let Ok(cfg) = config_clone.lock() {
                    monitor.check_drives(&cfg);
                }
            }
            
            thread::sleep(Duration::from_secs(2));
        }
    });
    
    // Start scheduled backup checker thread
    let config_clone2 = config.clone();
    thread::spawn(move || {
        loop {
            // Check if any scheduled backups need to run
            if let Ok(cfg) = config_clone2.lock() {
                cfg.check_scheduled_backups();
            }
            
            thread::sleep(Duration::from_secs(60));
        }
    });
    
    // Check for updates on startup
    log::info!("Checking for updates...");
    let config_clone3 = config.clone();
    thread::spawn(move || {
        thread::sleep(Duration::from_secs(5)); // Wait 5 seconds after startup
        
        if let Ok(cfg) = config_clone3.lock() {
            let checker = update_checker::UpdateChecker::new(&cfg);
            
            if checker.should_check_now() {
                if let Some(update_info) = checker.check_for_updates() {
                    if !checker.is_version_skipped(&update_info.version) {
                        log::info!("Update available: v{}", update_info.version);
                        update_notification::UpdateNotificationWindow::show(update_info, config_clone3.clone());
                    } else {
                        log::info!("Update v{} available but skipped by user", update_info.version);
                    }
                }
            }
        }
    });
    
    // Run the message loop
    nwg::dispatch_thread_events();
}