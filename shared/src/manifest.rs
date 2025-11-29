use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Complete update manifest from server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateManifest {
    pub latest_version: String,
    pub versions: HashMap<String, VersionInfo>,
}

/// Information about a specific version
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionInfo {
    pub release_date: String,
    pub breaking_changes: bool,
    pub min_compatible_version: String,
    
    // Download URLs
    pub download_url: String,
    pub checksum_sha256: String,
    pub changelog_url: String,
    pub file_size_bytes: u64,
    
    // Patch information
    #[serde(default)]
    pub has_patch: bool,
    #[serde(default)]
    pub patch_url: Option<String>,
    #[serde(default)]
    pub patch_checksum: Option<String>,
    #[serde(default)]
    pub patch_required_from: Vec<String>,
}

/// Update source configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateSource {
    pub name: String,
    pub url: String,
    pub enabled: bool,
    pub priority: u8, // 0 = highest
}

impl Default for UpdateSource {
    fn default() -> Self {
        Self {
            name: "GitHub".to_string(),
            url: "https://api.github.com/repos/ArsenijN/driveguard/releases".to_string(),
            enabled: true,
            priority: 0,
        }
    }
}

/// Update settings from config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateSettings {
    pub enabled: bool,
    pub check_frequency_days: u64,
    pub last_check: Option<String>,
    pub silent_updates: bool,
    pub wait_after_interaction_minutes: u64,
    pub auto_apply_patches: bool,
    pub skipped_versions: Vec<String>,
    pub allow_test_versions: bool, // Enable beta/RC versions
    pub sources: Vec<UpdateSource>,
}

impl Default for UpdateSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            check_frequency_days: 7,
            last_check: None,
            silent_updates: false,
            wait_after_interaction_minutes: 30,
            auto_apply_patches: true,
            skipped_versions: Vec::new(),
            allow_test_versions: false, // Disabled by default for stability
            sources: vec![
                UpdateSource {
                    name: "GitHub".to_string(),
                    url: "https://github.com/ArsenijN/driveguard/releases".to_string(),
                    enabled: true,
                    priority: 0,
                },
                UpdateSource {
                    name: "Custom Server".to_string(),
                    url: "https://arseniusgen.uk.to/projects/driveguard/manifest.json".to_string(),
                    enabled: true,
                    priority: 1,
                },
				UpdateSource {
                    name: "Custom Server".to_string(),
                    url: "http://arseniusgen.uk.to/projects/driveguard/manifest.json".to_string(),
                    enabled: true,
                    priority: 2,
                },
            ],
        }
    }
}

/// Parse semantic version string with optional release candidate suffix
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Version {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
    pub rc: Option<u32>, // Release candidate/test version number (e.g., r5, r137)
}

impl Version {
    pub fn parse(s: &str) -> Result<Self, String> {
        let s = s.trim_start_matches('v').trim_start_matches('V');
        
        // Check for release candidate suffix (e.g., "0.1.3r5")
        let (version_part, rc) = if let Some(r_pos) = s.find('r') {
            let (ver, rc_str) = s.split_at(r_pos);
            let rc_num = rc_str[1..].parse::<u32>()
                .map_err(|e| format!("Invalid release candidate number: {}", e))?;
            (ver, Some(rc_num))
        } else {
            (s, None)
        };
        
        let parts: Vec<&str> = version_part.split('.').collect();
        
        if parts.len() != 3 {
            return Err(format!("Invalid version format: {}", s));
        }
        
        Ok(Version {
            major: parts[0].parse().map_err(|e| format!("Invalid major version: {}", e))?,
            minor: parts[1].parse().map_err(|e| format!("Invalid minor version: {}", e))?,
            patch: parts[2].parse().map_err(|e| format!("Invalid patch version: {}", e))?,
            rc,
        })
    }
    
    pub fn to_string(&self) -> String {
        if let Some(rc) = self.rc {
            format!("{}.{}.{}r{}", self.major, self.minor, self.patch, rc)
        } else {
            format!("{}.{}.{}", self.major, self.minor, self.patch)
        }
    }
    
    pub fn is_stable(&self) -> bool {
        self.rc.is_none()
    }
    
    pub fn is_test(&self) -> bool {
        self.rc.is_some()
    }
    
    pub fn is_compatible_with(&self, other: &Version) -> bool {
        // Same major version = compatible
        // Test versions are compatible with their stable counterparts
        self.major == other.major
    }
    
    /// Get the base version without RC suffix (for comparison with stable)
    pub fn base_version(&self) -> Version {
        Version {
            major: self.major,
            minor: self.minor,
            patch: self.patch,
            rc: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_version_parsing() {
        let v1 = Version::parse("0.1.0").unwrap();
        let v2 = Version::parse("v0.1.3").unwrap();
        let v3 = Version::parse("0.2.0").unwrap();
        let v4 = Version::parse("0.1.3r5").unwrap();
        let v5 = Version::parse("v0.1.3r137").unwrap();
        
        assert!(v2 > v1);
        assert!(v3 > v2);
        assert!(v1.is_compatible_with(&v2));
        assert!(!v1.is_compatible_with(&v3));
        
        // Test version checks
        assert!(v1.is_stable());
        assert!(v4.is_test());
        assert_eq!(v4.rc, Some(5));
        assert_eq!(v5.rc, Some(137));
        
        // Test version strings
        assert_eq!(v4.to_string(), "0.1.3r5");
        assert_eq!(v5.to_string(), "0.1.3r137");
        
        // Test base version
        assert_eq!(v4.base_version(), Version::parse("0.1.3").unwrap());
    }
}