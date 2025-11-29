use std::collections::HashMap;
use std::sync::Mutex;
use lazy_static::lazy_static;

pub struct Localization {
    translations: HashMap<String, HashMap<String, String>>,
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
        // English translations
        let mut en = HashMap::new();
        en.insert("app_name".to_string(), "DriveGuard".to_string());
        en.insert("app_tagline".to_string(), "Automatic USB Drive Backup Tool".to_string());
        
        // Backup related
        en.insert("backup_starting".to_string(), "Backup for drive {0} is about to start".to_string());
        en.insert("backup_starting_in".to_string(), "Starting in {0}".to_string());
        en.insert("backup_in_progress".to_string(), "Backup in progress...".to_string());
        en.insert("backup_complete".to_string(), "Backup completed successfully!".to_string());
        en.insert("backup_failed".to_string(), "Backup failed".to_string());
        en.insert("backup_cancelled".to_string(), "Backup cancelled by user".to_string());
        en.insert("do_not_disconnect".to_string(), "Please do not disconnect the drive while backup is in progress".to_string());
        en.insert("files_copied".to_string(), "{0} files copied".to_string());
        
        // Buttons
        en.insert("button_cancel".to_string(), "Cancel".to_string());
        en.insert("button_hide".to_string(), "Hide".to_string());
        en.insert("button_start_now".to_string(), "Start Now".to_string());
        en.insert("button_ok".to_string(), "OK".to_string());
        en.insert("button_close".to_string(), "Close".to_string());
        
        // Menu items
        en.insert("menu_settings".to_string(), "Settings".to_string());
        en.insert("menu_schedules".to_string(), "View Schedules".to_string());
        en.insert("menu_about".to_string(), "About".to_string());
        en.insert("menu_exit".to_string(), "Exit".to_string());
        
        // Update notifications
        en.insert("update_available".to_string(), "DriveGuard Update Available!".to_string());
        en.insert("update_version_info".to_string(), "Version {0} is now available (you have {1})".to_string());
        en.insert("update_download_size".to_string(), "Download size: {0} MB".to_string());
        en.insert("update_changes".to_string(), "Changes:".to_string());
        en.insert("update_breaking_warning".to_string(), "This update contains breaking changes. Please review the changelog.".to_string());
        en.insert("update_compatible".to_string(), "This is a compatible update and can be installed safely.".to_string());
        en.insert("update_disable_info".to_string(), "To disable automatic updates, go to Settings > Updates".to_string());
        en.insert("button_update_now".to_string(), "Update Now".to_string());
        en.insert("button_ask_later".to_string(), "Ask Me Later".to_string());
        en.insert("button_skip_version".to_string(), "Skip This Version".to_string());
        en.insert("update_downloading".to_string(), "Downloading update...".to_string());
        en.insert("update_download_complete".to_string(), "Download complete! Applying update...".to_string());
        en.insert("update_failed".to_string(), "Update Failed".to_string());
        en.insert("update_download_failed".to_string(), "Download Failed".to_string());
        
        // Settings
        en.insert("settings_title".to_string(), "Settings".to_string());
        en.insert("settings_current".to_string(), "Current Settings:".to_string());
        en.insert("settings_language".to_string(), "Language:".to_string());
        en.insert("settings_min_space".to_string(), "Min Free Space:".to_string());
        en.insert("settings_warn_delete".to_string(), "Warn Before Delete:".to_string());
        en.insert("settings_active_schedules".to_string(), "Active Schedules:".to_string());
        en.insert("settings_edit_info".to_string(), "Edit 'settings.toml' to change settings.".to_string());
        
        // Schedules
        en.insert("schedules_title".to_string(), "Schedules".to_string());
        en.insert("schedules_none".to_string(), "No schedules configured yet.".to_string());
        en.insert("schedules_add_info".to_string(), "Add a schedule in settings.toml to get started!".to_string());
        en.insert("schedules_configured".to_string(), "Configured Schedules:".to_string());
        en.insert("schedule_enabled".to_string(), "Enabled".to_string());
        en.insert("schedule_disabled".to_string(), "Disabled".to_string());
        en.insert("schedule_interval".to_string(), "Interval: {0} days".to_string());
        en.insert("schedule_trigger_connect".to_string(), "Trigger on connect: {0}".to_string());
        en.insert("schedule_destination".to_string(), "Destination: {0}".to_string());
        
        // About
        en.insert("about_title".to_string(), "About DriveGuard".to_string());
        en.insert("about_version".to_string(), "DriveGuard v{0}".to_string());
        en.insert("about_features".to_string(), "Features:".to_string());
        en.insert("about_feature_detection".to_string(), "Drive detection by serial number".to_string());
        en.insert("about_feature_schedules".to_string(), "Schedule-based backups".to_string());
        en.insert("about_feature_copy".to_string(), "Full file copy with structure preservation".to_string());
        en.insert("about_created".to_string(), "Created with Rust 🦀".to_string());
        
        self.translations.insert("en".to_string(), en);
        
        // Ukrainian translations
        let mut uk = HashMap::new();
        uk.insert("app_name".to_string(), "DriveGuard".to_string());
        uk.insert("app_tagline".to_string(), "Автоматичне резервне копіювання USB-накопичувачів".to_string());
        
        // Backup related
        uk.insert("backup_starting".to_string(), "Резервне копіювання диска {0} розпочнеться".to_string());
        uk.insert("backup_starting_in".to_string(), "Початок через {0}".to_string());
        uk.insert("backup_in_progress".to_string(), "Виконується резервне копіювання...".to_string());
        uk.insert("backup_complete".to_string(), "Резервне копіювання успішно завершено!".to_string());
        uk.insert("backup_failed".to_string(), "Помилка резервного копіювання".to_string());
        uk.insert("backup_cancelled".to_string(), "Резервне копіювання скасовано користувачем".to_string());
        uk.insert("do_not_disconnect".to_string(), "⚠ Будь ласка, не від'єднуйте диск під час резервного копіювання".to_string());
        uk.insert("files_copied".to_string(), "Скопійовано файлів: {0}".to_string());
        
        // Buttons
        uk.insert("button_cancel".to_string(), "Скасувати".to_string());
        uk.insert("button_hide".to_string(), "Приховати".to_string());
        uk.insert("button_start_now".to_string(), "Почати зараз".to_string());
        uk.insert("button_ok".to_string(), "Гаразд".to_string());
        uk.insert("button_close".to_string(), "Закрити".to_string());
        
        // Menu items
        uk.insert("menu_settings".to_string(), "Налаштування".to_string());
        uk.insert("menu_schedules".to_string(), "Переглянути розклади".to_string());
        uk.insert("menu_about".to_string(), "Про програму".to_string());
        uk.insert("menu_exit".to_string(), "Вихід".to_string());
        
        // Update notifications
        uk.insert("update_available".to_string(), "🎉 Доступне оновлення DriveGuard!".to_string());
        uk.insert("update_version_info".to_string(), "Версія {0} тепер доступна (у вас {1})".to_string());
        uk.insert("update_download_size".to_string(), "Розмір завантаження: {0} МБ".to_string());
        uk.insert("update_changes".to_string(), "Зміни:".to_string());
        uk.insert("update_breaking_warning".to_string(), "⚠ Це оновлення містить критичні зміни. Будь ласка, перегляньте журнал змін.".to_string());
        uk.insert("update_compatible".to_string(), "Це сумісне оновлення і може бути встановлено безпечно.".to_string());
        uk.insert("update_disable_info".to_string(), "Щоб вимкнути автоматичні оновлення, перейдіть до Налаштування > Оновлення".to_string());
        uk.insert("button_update_now".to_string(), "Оновити зараз".to_string());
        uk.insert("button_ask_later".to_string(), "Запитати пізніше".to_string());
        uk.insert("button_skip_version".to_string(), "Пропустити цю версію".to_string());
        uk.insert("update_downloading".to_string(), "Завантаження оновлення...".to_string());
        uk.insert("update_download_complete".to_string(), "Завантаження завершено! Застосування оновлення...".to_string());
        uk.insert("update_failed".to_string(), "Помилка оновлення".to_string());
        uk.insert("update_download_failed".to_string(), "Помилка завантаження".to_string());
        
        // Settings
        uk.insert("settings_title".to_string(), "Налаштування".to_string());
        uk.insert("settings_current".to_string(), "Поточні налаштування:".to_string());
        uk.insert("settings_language".to_string(), "Мова:".to_string());
        uk.insert("settings_min_space".to_string(), "Мін. вільного місця:".to_string());
        uk.insert("settings_warn_delete".to_string(), "Попереджати перед видаленням:".to_string());
        uk.insert("settings_active_schedules".to_string(), "Активні розклади:".to_string());
        uk.insert("settings_edit_info".to_string(), "Відредагуйте 'settings.toml' для зміни налаштувань.".to_string());
        
        // Schedules
        uk.insert("schedules_title".to_string(), "Розклади".to_string());
        uk.insert("schedules_none".to_string(), "Ще не налаштовано жодного розкладу.".to_string());
        uk.insert("schedules_add_info".to_string(), "Додайте розклад у settings.toml, щоб почати!".to_string());
        uk.insert("schedules_configured".to_string(), "Налаштовані розклади:".to_string());
        uk.insert("schedule_enabled".to_string(), "Увімкнено".to_string());
        uk.insert("schedule_disabled".to_string(), "Вимкнено".to_string());
        uk.insert("schedule_interval".to_string(), "Інтервал: {0} днів".to_string());
        uk.insert("schedule_trigger_connect".to_string(), "Запуск при підключенні: {0}".to_string());
        uk.insert("schedule_destination".to_string(), "Призначення: {0}".to_string());
        
        // About
        uk.insert("about_title".to_string(), "Про DriveGuard".to_string());
        uk.insert("about_version".to_string(), "DriveGuard v{0}".to_string());
        uk.insert("about_features".to_string(), "Можливості:".to_string());
        uk.insert("about_feature_detection".to_string(), "• Виявлення дисків за серійним номером".to_string());
        uk.insert("about_feature_schedules".to_string(), "• Резервне копіювання за розкладом".to_string());
        uk.insert("about_feature_copy".to_string(), "• Повне копіювання файлів зі збереженням структури".to_string());
        uk.insert("about_created".to_string(), "Створено з Rust 🦀".to_string());
        
        self.translations.insert("uk".to_string(), uk);
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
        if let Some(locale_map) = self.translations.get(&self.current_locale) {
            if let Some(text) = locale_map.get(key) {
                return text.clone();
            }
        }
        
        // Fallback to English
        if let Some(locale_map) = self.translations.get("en") {
            if let Some(text) = locale_map.get(key) {
                return text.clone();
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