use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use chrono::{DateTime, Utc, Duration};
use driveguard_shared::manifest::UpdateSettings;

const CONFIG_FILE: &str = "settings.toml";
const SCHEDULES_DIR: &str = "schedules";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub general: GeneralSettings,
    pub schedules: Vec<BackupSchedule>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralSettings {
    #[serde(default = "default_language")]
    pub language: String,
    #[serde(default = "default_min_free_space")]
    pub min_free_space_gb: u64,
    #[serde(default = "default_true")]
    pub warn_before_delete: bool,
    #[serde(default)]
    pub update_settings: Option<UpdateSettings>,
}

// Default value functions for serde
fn default_language() -> String {
    "en".to_string()
}

fn default_min_free_space() -> u64 {
    10
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupSchedule {
    pub id: String,
    pub name: String,
    pub enabled: bool,
    
    // Drive identification
    pub drive_serial: Option<String>,
    pub drive_id_file: bool,
    
    // Backup settings
    pub source_paths: Vec<String>,
    pub destination_path: String,
    pub interval_days: u64,
    pub last_backup: Option<String>, // ISO 8601 format
    
    // Trigger settings
    pub trigger_on_connect: bool,
    pub trigger_on_schedule: bool,
    pub countdown_minutes: u64,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            general: GeneralSettings {
                language: "en".to_string(),
                min_free_space_gb: 10,
                warn_before_delete: true,
                update_settings: Some(UpdateSettings::default()),
            },
            schedules: Vec::new(),
        }
    }
}

impl AppConfig {
    pub fn load_or_create() -> Self {
        if Path::new(CONFIG_FILE).exists() {
            log::info!("Loading config from {}", CONFIG_FILE);
            let content = fs::read_to_string(CONFIG_FILE)
                .expect("Failed to read config file");
            
            log::info!("Config file content:\n{}", content);
            
            match toml::from_str::<Self>(&content) {
                Ok(mut config) => {
                    log::info!("Successfully parsed config with {} schedules", config.schedules.len());
                    
                    // Merge with defaults for any missing fields
                    if config.general.update_settings.is_none() {
                        log::info!("Update settings missing, using defaults");
                        config.general.update_settings = Some(UpdateSettings::default());
                        config.save(); // Save the updated config
                    }
                    
                    for schedule in &config.schedules {
                        log::info!("  - Schedule: {} (enabled: {})", schedule.name, schedule.enabled);
                    }
                    config
                }
                Err(e) => {
                    log::error!("Failed to parse config file: {}", e);
                    log::info!("Creating backup of invalid config and generating new one");
                    
                    // Backup the broken config
                    let backup_path = format!("{}.backup.{}", CONFIG_FILE, 
                                             chrono::Utc::now().format("%Y%m%d_%H%M%S"));
                    fs::copy(CONFIG_FILE, &backup_path).ok();
                    log::info!("Backed up invalid config to: {}", backup_path);
                    
                    let default = Self::default();
                    default.save();
                    default
                }
            }
        } else {
            log::info!("Config file not found, creating default");
            let config = Self::default();
            config.save();
            
            // Create schedules directory
            fs::create_dir_all(SCHEDULES_DIR).ok();
            
            config
        }
    }
    
    pub fn save(&self) {
        let content = toml::to_string_pretty(self)
            .expect("Failed to serialize config");
        fs::write(CONFIG_FILE, content)
            .expect("Failed to write config file");
    }
    
    pub fn add_schedule(&mut self, schedule: BackupSchedule) {
        self.schedules.push(schedule);
        self.save();
    }
    
    pub fn remove_schedule(&mut self, id: &str) {
        self.schedules.retain(|s| s.id != id);
        self.save();
    }
    
    pub fn update_last_backup(&mut self, schedule_id: &str) {
        if let Some(schedule) = self.schedules.iter_mut().find(|s| s.id == schedule_id) {
            schedule.last_backup = Some(Utc::now().to_rfc3339());
            self.save();
        }
    }
    
    pub fn check_scheduled_backups(&self) {
        let now = Utc::now();
        
        for schedule in &self.schedules {
            if !schedule.enabled || !schedule.trigger_on_schedule {
                continue;
            }
            
            let should_backup = if let Some(last_backup_str) = &schedule.last_backup {
                if let Ok(last_backup) = DateTime::parse_from_rfc3339(last_backup_str) {
                    let elapsed = now.signed_duration_since(last_backup);
                    elapsed >= Duration::days(schedule.interval_days as i64)
                } else {
                    true
                }
            } else {
                true // Never backed up before
            };
            
            if should_backup {
                log::info!("Schedule '{}' is due for backup", schedule.name);
                // TODO: Trigger backup countdown window
            }
        }
    }
}

impl BackupSchedule {
    pub fn new(name: String) -> Self {
        use std::time::{SystemTime, UNIX_EPOCH};
        
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        Self {
            id: format!("schedule_{}", timestamp),
            name,
            enabled: true,
            drive_serial: None,
            drive_id_file: true,
            source_paths: Vec::new(),
            destination_path: String::new(),
            interval_days: 7,
            last_backup: None,
            trigger_on_connect: true,
            trigger_on_schedule: false,
            countdown_minutes: 5,
        }
    }
    
    pub fn load_backup_list(&self) -> Vec<String> {
        let list_file = format!("{}/{}_backup_list.txt", SCHEDULES_DIR, self.id);
        
        if Path::new(&list_file).exists() {
            fs::read_to_string(&list_file)
                .unwrap_or_default()
                .lines()
                .filter(|line| !line.trim().is_empty() && !line.trim().starts_with('#'))
                .map(|s| s.to_string())
                .collect()
        } else {
            // Create default backup list file with instructions
            let default_content = r#"# DriveGuard Backup List
# Add one path per line. Examples:
# C:\Users\YourName\Documents
# C:\Users\YourName\Pictures
# D:\ImportantData

"#;
            fs::write(&list_file, default_content).ok();
            Vec::new()
        }
    }
    
    pub fn save_backup_list(&self, paths: &[String]) {
        let list_file = format!("{}/{}_backup_list.txt", SCHEDULES_DIR, self.id);
        let content = paths.join("\n");
        fs::write(&list_file, content).ok();
    }
}