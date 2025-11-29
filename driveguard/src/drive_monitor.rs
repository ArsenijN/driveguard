use std::collections::HashMap;
use std::fs;
use std::path::Path;
use windows::Win32::Storage::FileSystem::{
    GetVolumeInformationW, GetLogicalDrives, GetDriveTypeW,
};
use windows::core::PWSTR;
use crate::config::AppConfig;

const DRIVE_ID_FILE: &str = ".driveGuardID";

#[derive(Debug, Clone)]
pub struct DriveInfo {
    pub letter: char,
    pub serial: Option<u32>,
    pub has_id_file: bool,
    pub id_content: Option<String>,
}

#[derive(Default)]
pub struct DriveMonitor {
    connected_drives: HashMap<char, DriveInfo>,
}

impl DriveMonitor {
    pub fn new() -> Self {
        Self {
            connected_drives: HashMap::new(),
        }
    }
    
    pub fn check_drives(&mut self, config: &AppConfig) {
        let current_drives = Self::get_all_drives();
        
        // Check for newly connected drives
        for (letter, info) in &current_drives {
            if !self.connected_drives.contains_key(letter) {
                log::info!("Drive {} connected", letter);
                self.on_drive_connected(*letter, info, config);
            }
        }
        
        // Check for disconnected drives
        let disconnected: Vec<char> = self.connected_drives
            .keys()
            .filter(|k| !current_drives.contains_key(k))
            .copied()
            .collect();
        
        for letter in disconnected {
            log::info!("Drive {} disconnected", letter);
            self.connected_drives.remove(&letter);
        }
        
        self.connected_drives = current_drives;
    }
    
    // Check all currently connected drives on startup
    pub fn check_all_drives_on_startup(&mut self, config: &AppConfig) {
        let current_drives = Self::get_all_drives();
        
        for (letter, info) in &current_drives {
            log::info!("Checking existing drive {} on startup", letter);
            self.on_drive_connected(*letter, info, config);
        }
        
        self.connected_drives = current_drives;
    }
    
    fn on_drive_connected(&self, letter: char, info: &DriveInfo, config: &AppConfig) {
        log::info!("Checking drive {} against {} schedules", letter, config.schedules.len());
        
        // Check if any schedule matches this drive
        for schedule in &config.schedules {
            log::info!("Checking schedule '{}' (enabled: {}, trigger_on_connect: {})", 
                      schedule.name, schedule.enabled, schedule.trigger_on_connect);
            
            if !schedule.enabled || !schedule.trigger_on_connect {
                log::info!("  Skipping schedule '{}' - not enabled or trigger_on_connect is false", schedule.name);
                continue;
            }
            
            let matches = if let Some(ref target_serial) = schedule.drive_serial {
                if !target_serial.is_empty() {
                    // Check by serial number
                    log::info!("  Checking by serial number: target='{}', drive={:?}", target_serial, info.serial);
                    if let Some(drive_serial) = info.serial {
                        let matches = target_serial == &drive_serial.to_string();
                        log::info!("  Serial match result: {}", matches);
                        matches
                    } else {
                        log::info!("  Drive has no serial number");
                        false
                    }
                } else {
                    log::info!("  Serial is empty, checking ID file instead");
                    schedule.drive_id_file && info.has_id_file
                }
            } else if schedule.drive_id_file {
                // Check by ID file
                log::info!("  Checking by ID file: has_id_file={}", info.has_id_file);
                info.has_id_file
            } else {
                log::info!("  No matching criteria configured");
                false
            };
            
            if matches {
                log::info!("✓ Drive matches schedule '{}'", schedule.name);
                self.check_and_trigger_backup(schedule, letter);
            } else {
                log::info!("✗ Drive does NOT match schedule '{}'", schedule.name);
            }
        }
    }
    
    fn check_and_trigger_backup(&self, schedule: &crate::config::BackupSchedule, drive_letter: char) {
        use chrono::{DateTime, Utc, Duration};
        
        log::info!("==> check_and_trigger_backup CALLED for drive {} and schedule '{}'", drive_letter, schedule.name);
        
        let now = Utc::now();
        let should_backup = if let Some(ref last_backup_str) = schedule.last_backup {
            if !last_backup_str.is_empty() {
                if let Ok(last_backup) = DateTime::parse_from_rfc3339(last_backup_str) {
                    let elapsed = now.signed_duration_since(last_backup);
                    elapsed >= Duration::days(schedule.interval_days as i64)
                } else {
                    true
                }
            } else {
                true // Empty string means never backed up
            }
        } else {
            true // None means never backed up
        };
        
        log::info!("==> Should backup: {}", should_backup);
        
        if should_backup {
            log::info!("==> Backup is due for schedule '{}', CALLING CountdownWindow::show", schedule.name);
            crate::countdown_window::CountdownWindow::show(schedule.clone(), drive_letter);
            log::info!("==> CountdownWindow::show returned");
        } else {
            log::info!("Backup not due yet for schedule '{}'", schedule.name);
        }
    }
    
    fn get_all_drives() -> HashMap<char, DriveInfo> {
        let mut drives = HashMap::new();
        
        unsafe {
            let bitmask = GetLogicalDrives();
            
            for i in 0..26 {
                if (bitmask & (1 << i)) != 0 {
                    let letter = (b'A' + i) as char;
                    let drive_path = format!("{}:\\", letter);
                    
                    // Check if it's a removable or fixed drive
                    let drive_type = {
                        let mut path_wide: Vec<u16> = drive_path.encode_utf16().collect();
                        path_wide.push(0);
                        GetDriveTypeW(PWSTR(path_wide.as_mut_ptr()))
                    };
                    
                    // 2 = removable, 3 = fixed
                    if drive_type == 2 || drive_type == 3 {
                        let serial = Self::get_volume_serial(&drive_path);
                        let (has_id_file, id_content) = Self::check_id_file(&drive_path);
                        
                        log::info!("Drive {} - Serial: {:?}, Has ID file: {}, ID content: {:?}", 
                                  letter, serial, has_id_file, id_content);
                        
                        drives.insert(letter, DriveInfo {
                            letter,
                            serial,
                            has_id_file,
                            id_content,
                        });
                    }
                }
            }
        }
        
        drives
    }
    
    fn get_volume_serial(drive_path: &str) -> Option<u32> {
        unsafe {
            let mut path_wide: Vec<u16> = drive_path.encode_utf16().collect();
            path_wide.push(0);
            
            let mut serial: u32 = 0;
            let mut max_component_len: u32 = 0;
            let mut file_system_flags: u32 = 0;
            
            let result = GetVolumeInformationW(
                PWSTR(path_wide.as_mut_ptr()),
                None,
                Some(&mut serial),
                Some(&mut max_component_len),
                Some(&mut file_system_flags),
                None,
            );
            
            if result.is_ok() {
                Some(serial)
            } else {
                None
            }
        }
    }
    
    fn check_id_file(drive_path: &str) -> (bool, Option<String>) {
        let id_file_path = format!("{}{}", drive_path, DRIVE_ID_FILE);
        
        if Path::new(&id_file_path).exists() {
            let content = fs::read_to_string(&id_file_path).ok();
            (true, content)
        } else {
            (false, None)
        }
    }
    
    pub fn create_id_file(drive_path: &str, id: &str) -> std::io::Result<()> {
        let id_file_path = format!("{}{}", drive_path, DRIVE_ID_FILE);
        fs::write(&id_file_path, id)
    }
}