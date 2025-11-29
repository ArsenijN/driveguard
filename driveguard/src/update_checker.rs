use std::process::Command;
use std::thread;
use std::time::Duration;
use chrono::{DateTime, Utc};
use driveguard_shared::manifest::UpdateSettings;
use crate::config::AppConfig;

// Get version from version.rs module
pub fn get_current_version() -> &'static str {
    crate::version::VERSION
}

pub struct UpdateChecker {
    settings: UpdateSettings,
    last_interaction: Option<DateTime<Utc>>,
}

impl UpdateChecker {
    pub fn new(config: &AppConfig) -> Self {
        // Load update settings from config
        let settings = config.general.update_settings.clone().unwrap_or_default();
        
        Self {
            settings,
            last_interaction: None,
        }
    }
    
    pub fn should_check_now(&self) -> bool {
        if !self.settings.enabled {
            return false;
        }
        
        // Check if enough time has passed since last check
        if let Some(ref last_check_str) = self.settings.last_check {
            if let Ok(last_check) = DateTime::parse_from_rfc3339(last_check_str) {
                let elapsed = Utc::now().signed_duration_since(last_check);
                let days = elapsed.num_days();
                
                if days < self.settings.check_frequency_days as i64 {
                    log::info!("Update check not due yet ({} days since last check)", days);
                    return false;
                }
            }
        }
        
        true
    }
    
    pub fn check_for_updates(&self) -> Option<UpdateInfo> {
        log::info!("Checking for updates...");
        
        // Sort sources by priority
        let mut sources = self.settings.sources.clone();
        sources.sort_by_key(|s| s.priority);
        
        // Try each source in order
        for source in sources.iter().filter(|s| s.enabled) {
            log::info!("Trying update source: {} ({})", source.name, source.url);
            
            match self.check_source(&source.url) {
                Ok(info) => {
                    log::info!("Found update from {}: v{}", source.name, info.version);
                    return Some(info);
                }
                Err(e) => {
                    log::warn!("Failed to check {}: {}", source.name, e);
                    continue;
                }
            }
        }
        
        log::info!("No updates available from any source");
        None
    }
    
    fn check_source(&self, manifest_url: &str) -> Result<UpdateInfo, String> {
        // Try to find updater.exe in multiple locations
        let updater_paths = vec![
            "updater.exe",
            "./updater.exe",
            "../updater/target/debug/updater.exe",
            "../updater/target/release/updater.exe",
        ];
        
        let mut updater_found = false;
        let mut last_error = String::new();
        
        for updater_path in updater_paths {
            // Call updater to check for updates
            match Command::new(updater_path)
                .arg("--check")
                .arg(manifest_url)
                .arg(get_current_version())
                .output()
            {
                Ok(output) => {
                    updater_found = true;
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    
                    log::debug!("Updater stdout: {}", stdout);
                    if !stderr.is_empty() {
                        log::debug!("Updater stderr: {}", stderr);
                    }
                    
                    // Parse updater output
                    for line in stdout.lines() {
                        if line.starts_with("UPDATE_AVAILABLE:") {
                            let version = line.strip_prefix("UPDATE_AVAILABLE:").unwrap().to_string();
                            
                            // Check if it's a test version and if user allows them
                            let is_test_version = version.contains('r');
                            if is_test_version && !self.settings.allow_test_versions {
                                log::info!("Skipping test version {} (test versions disabled)", version);
                                return Err("Test version not allowed".to_string());
                            }
                            
                            // Parse additional info from following lines
                            let mut url = String::new();
                            let mut checksum = String::new();
                            let mut size = 0u64;
                            let mut breaking = false;
                            
                            for info_line in stdout.lines() {
                                if info_line.starts_with("URL:") {
                                    url = info_line.strip_prefix("URL:").unwrap().to_string();
                                } else if info_line.starts_with("CHECKSUM:") {
                                    checksum = info_line.strip_prefix("CHECKSUM:").unwrap().to_string();
                                } else if info_line.starts_with("SIZE:") {
                                    size = info_line.strip_prefix("SIZE:").unwrap().parse().unwrap_or(0);
                                } else if info_line.starts_with("BREAKING:") {
                                    breaking = info_line.strip_prefix("BREAKING:").unwrap() == "true";
                                }
                            }
                            
                            return Ok(UpdateInfo {
                                version,
                                url,
                                checksum,
                                size_bytes: size,
                                breaking_changes: breaking,
                            });
                        } else if line == "UP_TO_DATE" {
                            return Err("Already up to date".to_string());
                        }
                    }
                    
                    if !stdout.is_empty() {
                        log::error!("Updater output did not contain expected markers. Got {} bytes", stdout.len());
                    }
                    return Err("Failed to parse updater output".to_string());
                }
                Err(e) => {
                    last_error = format!("{}", e);
                    log::debug!("Failed to run updater at {}: {}", updater_path, e);
                    continue;
                }
            }
        }
        
        if !updater_found {
            log::warn!("Updater executable not found in any expected location. Last error: {}", last_error);
        }
        Err("Failed to find or execute updater".to_string())
    }
    
    pub fn download_update(&self, info: &UpdateInfo) -> Result<String, String> {
        log::info!("Downloading update v{}...", info.version);
        
        let output = Command::new("updater.exe")
            .arg("--download")
            .arg(&info.version)
            .arg(&info.url)
            .arg(&info.checksum)
            .output()
            .map_err(|e| format!("Failed to run updater: {}", e))?;
        
        let stdout = String::from_utf8_lossy(&output.stdout);
        
        for line in stdout.lines() {
            if line.starts_with("DOWNLOAD_COMPLETE:") {
                let path = line.strip_prefix("DOWNLOAD_COMPLETE:").unwrap().to_string();
                log::info!("Download complete: {}", path);
                return Ok(path);
            }
        }
        
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(format!("Download failed: {}", stderr))
    }
    
    pub fn apply_update(&self, version: &str) -> Result<(), String> {
        log::info!("Applying update v{}...", version);
        
        // Start updater to apply update
        Command::new("updater.exe")
            .arg("--apply")
            .arg(version)
            .arg(get_current_version())
            .spawn()
            .map_err(|e| format!("Failed to start updater: {}", e))?;
        
        // Exit DriveGuard so updater can replace the executable
        log::info!("Exiting to apply update...");
        std::process::exit(0);
    }
    
    pub fn update_last_interaction(&mut self) {
        self.last_interaction = Some(Utc::now());
    }
    
    pub fn should_apply_silent_update(&self) -> bool {
        if !self.settings.silent_updates {
            return false;
        }
        
        // Check if enough time has passed since last interaction
        if let Some(last_interaction) = self.last_interaction {
            let elapsed = Utc::now().signed_duration_since(last_interaction);
            let minutes = elapsed.num_minutes();
            
            if minutes >= self.settings.wait_after_interaction_minutes as i64 {
                return true;
            }
        }
        
        false
    }
    
    pub fn is_version_skipped(&self, version: &str) -> bool {
        self.settings.skipped_versions.contains(&version.to_string())
    }
}

#[derive(Debug, Clone)]
pub struct UpdateInfo {
    pub version: String,
    pub url: String,
    pub checksum: String,
    pub size_bytes: u64,
    pub breaking_changes: bool,
}

pub fn start_update_checker_thread(config: std::sync::Arc<std::sync::Mutex<AppConfig>>) {
    thread::spawn(move || {
        loop {
            thread::sleep(Duration::from_secs(3600)); // Check every hour
            
            if let Ok(cfg) = config.lock() {
                let checker = UpdateChecker::new(&cfg);
                
                if checker.should_check_now() {
                    if let Some(update_info) = checker.check_for_updates() {
                        log::info!("Update available: v{}", update_info.version);
                        
                        // TODO: Show notification to user
                        // This will be integrated with the UI
                    }
                }
            }
        }
    });
}