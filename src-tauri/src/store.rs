use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::PathBuf;

const INACTIVITY_DEFAULT_MS: u64 = 5 * 60 * 1000;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bookmark {
    pub id: String,
    pub title: String,
    pub url: String,
    pub partition: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub badge: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
    #[serde(rename = "iconSvg", skip_serializing_if = "Option::is_none")]
    pub icon_svg: Option<String>,
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    #[serde(rename = "lockEnabled")]
    pub lock_enabled: bool,
    #[serde(rename = "inactivityMs")]
    pub inactivity_ms: u64,
    #[serde(rename = "pinSalt")]
    pub pin_salt: Option<String>,
    #[serde(rename = "pinHash")]
    pub pin_hash: Option<String>,
    #[serde(rename = "lockOnLaunch", default = "default_true")]
    pub lock_on_launch: bool,
    #[serde(rename = "startMinimized", default)]
    pub start_minimized: bool,
}

impl Default for Settings {
    fn default() -> Self {
        let mut s = Self {
            lock_enabled: true,
            inactivity_ms: INACTIVITY_DEFAULT_MS,
            pin_salt: None,
            pin_hash: None,
            lock_on_launch: true,
            start_minimized: false,
        };
        s.set_pin("123456");
        s
    }
}

impl Settings {
    pub fn set_pin(&mut self, pin: &str) {
        let mut salt_bytes = [0u8; 16];
        let _ = getrandom::getrandom(&mut salt_bytes);
        let salt = hex::encode(salt_bytes);
        let hash = sha256_hash(pin, &salt);
        self.pin_salt = Some(salt);
        self.pin_hash = Some(hash);
    }

    pub fn verify_pin(&self, pin: &str) -> bool {
        match (&self.pin_salt, &self.pin_hash) {
            (Some(salt), Some(hash)) => sha256_hash(pin, salt) == *hash,
            _ => false,
        }
    }
}

pub fn sha256_hash(pin: &str, salt: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(format!("{}:{}", pin, salt).as_bytes());
    hex::encode(hasher.finalize())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppState {
    pub settings: Settings,
    pub bookmarks: Vec<Bookmark>,
}

pub struct StoreManager {
    base_dir: PathBuf,
}

impl StoreManager {
    pub fn new(base_dir: PathBuf) -> Self {
        let _ = fs::create_dir_all(&base_dir);
        Self { base_dir }
    }

    pub fn settings_path(&self) -> PathBuf {
        self.base_dir.join("settings.json")
    }

    pub fn bookmarks_path(&self) -> PathBuf {
        self.base_dir.join("bookmarks.json")
    }

    pub fn sessions_dir(&self) -> PathBuf {
        let dir = self.base_dir.join("sessions");
        let _ = fs::create_dir_all(&dir);
        dir
    }

    pub fn session_data_dir(&self, partition: &str) -> PathBuf {
        let safe_name: String = partition
            .chars()
            .map(|c| if c.is_alphanumeric() || c == '_' || c == '-' { c } else { '_' })
            .collect();
        let dir = self.sessions_dir().join(safe_name);
        let _ = fs::create_dir_all(&dir);
        dir
    }

    pub fn load_settings(&self) -> Settings {
        let p = self.settings_path();
        if let Ok(content) = fs::read_to_string(&p) {
            if let Ok(settings) = serde_json::from_str::<Settings>(&content) {
                return settings;
            }
        }
        let default_settings = Settings::default();
        self.save_settings(&default_settings);
        default_settings
    }

    pub fn save_settings(&self, settings: &Settings) {
        if let Ok(json) = serde_json::to_string_pretty(settings) {
            let _ = fs::write(self.settings_path(), json);
        }
    }

    pub fn load_bookmarks(&self) -> Vec<Bookmark> {
        let p = self.bookmarks_path();
        if let Ok(content) = fs::read_to_string(&p) {
            if let Ok(bookmarks) = serde_json::from_str::<Vec<Bookmark>>(&content) {
                return bookmarks;
            }
        }

        // Try migrating from existing Electron configuration if present!
        if let Some(migrated) = try_migrate_electron_bookmarks() {
            self.save_bookmarks(&migrated);
            return migrated;
        }

        Vec::new()
    }

    pub fn save_bookmarks(&self, bookmarks: &[Bookmark]) {
        if let Ok(json) = serde_json::to_string_pretty(bookmarks) {
            let _ = fs::write(self.bookmarks_path(), json);
        }
    }
}

fn try_migrate_electron_bookmarks() -> Option<Vec<Bookmark>> {
    #[cfg(target_os = "macos")]
    let base = dirs::data_dir()
        .map(|d| d.join("com.larico.minibrowser"))
        .or_else(|| {
            dirs::home_dir().map(|h| h.join("Library/Application Support/com.larico.minibrowser"))
        });

    #[cfg(not(target_os = "macos"))]
    let base = dirs::data_dir().map(|d| d.join("com.larico.minibrowser"));

    if let Some(dir) = base {
        let path = dir.join("bookmarks.json");
        if path.exists() {
            if let Ok(raw) = fs::read_to_string(&path) {
                #[derive(Deserialize)]
                struct OldBookmark {
                    title: Option<String>,
                    url: String,
                    partition: String,
                    #[serde(rename = "iconSvg")]
                    icon_svg: Option<String>,
                }
                if let Ok(old) = serde_json::from_str::<Vec<OldBookmark>>(&raw) {
                    let new_list: Vec<Bookmark> = old
                        .into_iter()
                        .enumerate()
                        .map(|(i, b)| Bookmark {
                            id: format!("bm_{}", i + 1),
                            title: b.title.unwrap_or_else(|| b.url.clone()),
                            url: b.url,
                            partition: b.partition,
                            badge: None,
                            color: None,
                            icon_svg: b.icon_svg,
                        })
                        .collect();
                    return Some(new_list);
                }
            }
        }
    }
    None
}
