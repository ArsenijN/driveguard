use std::collections::HashMap;
use std::sync::Mutex;
use lazy_static::lazy_static;
use serde_json::{json, Value};
use std::path::PathBuf;

pub struct Localization {
    translations: HashMap<String, Value>,
    current_locale: String,
}

impl Localization {
    pub fn new(locale: &str) -> Self {
        let mut loc = Self {
            translations: HashMap::new(),
            current_locale: locale.to_string(),
        };
        
        loc.load_all_translations();
        loc
    }
    
    fn load_all_translations(&mut self) {
        // Try to load from assets/locales/ directory
        let locales = vec!["en", "uk"];
        
        for lang in locales {
            // Try to load from file first
            let file_content = self.try_load_from_file(lang);
            
            if let Some(content) = file_content {
                if let Ok(json) = serde_json::from_str::<Value>(&content) {
                    self.translations.insert(lang.to_string(), json);
                    log::info!("Loaded locale '{}' from file", lang);
                    continue;
                }
            }
            
            // Fallback: load embedded defaults if file not found
            log::warn!("Loading embedded default for locale '{}'", lang);
            if let Some(default_json) = self.get_embedded_default(lang) {
                self.translations.insert(lang.to_string(), default_json);
            }
        }
        
        // Ensure at least English is available
        if !self.translations.contains_key("en") {
            log::error!("English locale failed to load!");
            self.translations.insert("en".to_string(), json!({}));
        }
    }
    
    fn try_load_from_file(&self, lang: &str) -> Option<String> {
        // Try multiple paths:
        // 1. assets/locales/{lang}.json (dev/release from project root)
        // 2. locales/{lang}.json (relative to executable)
        // 3. {exe_dir}/locales/{lang}.json (beside executable)
        
        let paths = vec![
            PathBuf::from(format!("assets/locales/{}.json", lang)),
            PathBuf::from(format!("locales/{}.json", lang)),
            {
                if let Ok(exe_path) = std::env::current_exe() {
                    if let Some(parent) = exe_path.parent() {
                        parent.join(format!("locales/{}.json", lang))
                    } else {
                        PathBuf::from("")
                    }
                } else {
                    PathBuf::from("")
                }
            },
        ];
        
        for path in paths {
            if path.as_os_str().is_empty() {
                continue;
            }
            if let Ok(content) = std::fs::read_to_string(&path) {
                log::info!("Loaded locale from: {:?}", path);
                return Some(content);
            }
        }
        
        None
    }
    
    fn get_embedded_default(&self, lang: &str) -> Option<Value> {
        // Embedded fallback translations (minimal, for dev/testing)
        match lang {
            "en" => Some(json!({
                "app_name": "DriveGuard",
                "app_tagline": "Automatic USB Drive Backup Tool",
                "backup_in_progress": "Backup in progress...",
                "button_cancel": "Cancel",
                "button_ok": "OK",
            })),
            "uk" => Some(json!({
                "app_name": "DriveGuard",
                "app_tagline": "Автоматичне резервне копіювання USB-накопичувачів",
                "backup_in_progress": "Виконується резервне копіювання...",
                "button_cancel": "Скасувати",
                "button_ok": "Гаразд",
            })),
            _ => None,
        }
    }
    
    pub fn set_locale(&mut self, locale: &str) {
        if self.translations.contains_key(locale) {
            self.current_locale = locale.to_string();
            log::info!("Locale changed to: {}", locale);
        } else {
            log::warn!("Locale '{}' not found, using default", locale);
        }
    }
    
    pub fn get(&self, key: &str) -> String {
        // Try current locale
        if let Some(locale_obj) = self.translations.get(&self.current_locale) {
            if let Some(value) = locale_obj.get(key) {
                if let Some(s) = value.as_str() {
                    return s.to_string();
                }
            }
        }
        
        // Fallback to English
        if let Some(locale_obj) = self.translations.get("en") {
            if let Some(value) = locale_obj.get(key) {
                if let Some(s) = value.as_str() {
                    return s.to_string();
                }
            }
        }
        
        format!("[Missing: {}]", key)
    }
    
    pub fn get_formatted(&self, key: &str, args: &[&str]) -> String {
        let mut text = self.get(key);
        
        for (i, arg) in args.iter().enumerate() {
            text = text.replace(&format!("{{{}}}", i), arg);
        }
        
        text
    }
}

// Global localization instance
lazy_static! {
    pub static ref LOC: Mutex<Localization> = Mutex::new(Localization::new("en"));
}

pub fn t(key: &str) -> String {
    LOC.lock().unwrap().get(key)
}

pub fn tf(key: &str, args: &[&str]) -> String {
    LOC.lock().unwrap().get_formatted(key, args)
}

pub fn set_locale(locale: &str) {
    LOC.lock().unwrap().set_locale(locale);
}