use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;
use chrono::Utc;
use std::collections::HashMap;

pub struct BackupEngine {
    pub total_files: usize,
    pub copied_files: usize,
    pub failed_files: Vec<(String, String)>, // (path, error)
    pub is_running: bool,
}

impl BackupEngine {
    pub fn new() -> Self {
        Self {
            total_files: 0,
            copied_files: 0,
            failed_files: Vec::new(),
            is_running: false,
        }
    }
    
    pub fn run_backup(
        &mut self,
        source_paths: &[String],
        destination_base: &str,
    ) -> Result<String, String> {
        self.is_running = true;
        self.total_files = 0;
        self.copied_files = 0;
        self.failed_files.clear();
        
        // Create timestamped backup folder (ISO 8601, NTFS-safe)
        let timestamp = Utc::now().format("%Y-%m-%dT%H-%M-%S").to_string();
        let backup_folder = format!("{}\\{}", destination_base, timestamp);
        
        fs::create_dir_all(&backup_folder)
            .map_err(|e| format!("Failed to create backup folder: {}", e))?;
        
        // Track folder names to avoid duplicates
        let mut folder_counter: HashMap<String, u32> = HashMap::new();
        
        // Process each source path
        for source in source_paths {
            let source_path = Path::new(source);
            
            if !source_path.exists() {
                log::warn!("Source path does not exist: {}", source);
                continue;
            }
            
            // Extract the folder name
            let folder_name = if let Some(name) = source_path.file_name() {
                name.to_string_lossy().to_string()
            } else {
                // Handle drive roots like C:\
                source_path.to_string_lossy()
                    .trim_end_matches(":\\")
                    .to_string()
            };
            
            // Check for duplicate folder names
            let final_folder_name = if let Some(count) = folder_counter.get(&folder_name) {
                let new_count = count + 1;
                folder_counter.insert(folder_name.clone(), new_count);
                format!("{}_{}", folder_name, new_count)
            } else {
                folder_counter.insert(folder_name.clone(), 0);
                folder_name
            };
            
            let dest_folder = format!("{}\\{}", backup_folder, final_folder_name);
            
            // Copy the directory tree
            self.copy_directory(source_path, Path::new(&dest_folder))?;
        }
        
        self.is_running = false;
        Ok(backup_folder)
    }
    
    fn copy_directory(&mut self, source: &Path, destination: &Path) -> Result<(), String> {
        // Create destination directory
        fs::create_dir_all(destination)
            .map_err(|e| format!("Failed to create directory {}: {}", destination.display(), e))?;
        
        // Walk through source directory
        for entry in WalkDir::new(source).into_iter().filter_map(|e| e.ok()) {
            let path = entry.path();
            
            if path == source {
                continue;
            }
            
            // Calculate relative path
            let relative = path.strip_prefix(source)
                .map_err(|e| format!("Failed to strip prefix: {}", e))?;
            
            let dest_path = destination.join(relative);
            
            if entry.file_type().is_dir() {
                // Create directory
                if let Err(e) = fs::create_dir_all(&dest_path) {
                    log::warn!("Failed to create directory {}: {}", dest_path.display(), e);
                }
            } else {
                // Copy file
                self.total_files += 1;
                
                // Ensure parent directory exists
                if let Some(parent) = dest_path.parent() {
                    fs::create_dir_all(parent).ok();
                }
                
                match fs::copy(path, &dest_path) {
                    Ok(_) => {
                        self.copied_files += 1;
                    }
                    Err(e) => {
                        let error_msg = format!("{}", e);
                        self.failed_files.push((
                            path.to_string_lossy().to_string(),
                            error_msg,
                        ));
                        log::warn!("Failed to copy {}: {}", path.display(), e);
                    }
                }
            }
        }
        
        Ok(())
    }
    
    pub fn get_progress(&self) -> (usize, usize) {
        (self.copied_files, self.total_files)
    }
    
    pub fn save_logs(&self, backup_folder: &str) -> std::io::Result<()> {
        // Save backup log
        let mut log_content = String::from("DriveGuard Backup Log\n");
        log_content.push_str(&format!("Timestamp: {}\n", Utc::now().to_rfc3339()));
        log_content.push_str(&format!("Total files: {}\n", self.total_files));
        log_content.push_str(&format!("Successfully copied: {}\n", self.copied_files));
        log_content.push_str(&format!("Failed: {}\n\n", self.failed_files.len()));
        
        for (path, _) in &self.failed_files {
            log_content.push_str(&format!("{} - OK\n", path));
        }
        
        let log_path = format!("{}\\backup.txt", backup_folder);
        fs::write(&log_path, log_content)?;
        
        // Save error log if there are failures
        if !self.failed_files.is_empty() {
            let mut error_content = String::from("DriveGuard Backup Errors\n\n");
            
            for (path, error) in &self.failed_files {
                error_content.push_str(&format!("{} - Failed! ({})\n", path, error));
            }
            
            let error_path = format!("{}\\backup_errors.txt", backup_folder);
            fs::write(&error_path, error_content)?;
        }
        
        Ok(())
    }
}
