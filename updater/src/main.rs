// DriveGuard Updater
// Handles downloading and applying updates

use std::env;
use std::fs;
use std::path::{PathBuf};
use std::process::Command;
use sha2::{Sha256, Digest};
use driveguard_shared::manifest::{UpdateManifest, Version};

// Embedded CA certificate (self-signed, safe to include in source)
const CUSTOM_CA_CERT: &[u8] = br#"-----BEGIN CERTIFICATE-----
MIIEKzCCAxOgAwIBAgIUVMekl1/DQZ9oWXOGXIRNc9OxazIwDQYJKoZIhvcNAQEL
BQAwgaQxCzAJBgNVBAYTAlVBMRMwEQYDVQQIDApTb21lLVN0YXRlMREwDwYDVQQH
DAhVemhob3JvZDEeMBwGA1UECgwVYXJzZW5pdXMuZ2VuIEhvc3RpbmdzMRAwDgYD
VQQLDAdIb3N0aW5nMRAwDgYDVQQDDAdBcnNlbmlpMSkwJwYJKoZIhvcNAQkBFhph
cnNlbmlqbm9jaGV2bnlqQGdtYWlsLmNvbTAeFw0yNTA3MDkwMDA4MTJaFw0yNjA3
MDkwMDA4MTJaMIGkMQswCQYDVQQGEwJVQTETMBEGA1UECAwKU29tZS1TdGF0ZTER
MA8GA1UEBwwIVXpoaG9yb2QxHjAcBgNVBAoMFWFyc2VuaXVzLmdlbiBIb3N0aW5n
czEQMA4GA1UECwwHSG9zdGluZzEQMA4GA1UEAwwHQXJzZW5paTEpMCcGCSqGSIb3
DQEJARYaYXJzZW5pam5vY2hldm55akBnbWFpbC5jb20wggEiMA0GCSqGSIb3DQEB
AQUAA4IBDwAwggEKAoIBAQCz68hDSrhvXWEdBxeDGmNa+uCa36rk2rowA9UKsK/s
6QFkEnCwTEoXdjVdURBlvs+/+ht456A/PyMUJ7FV17tboxGXDir9W9hNbfa4P29m
3fRz7aWq0yGOkoJNoTxu49cmHp9oex0A6eNH/UtwalniTsJCtDPR491bw7TlBVnV
mn6YnspM8SrfGIe9RtFf52Q1KLM/r1JGp5xYlO65Z1frX80s12eO3ov10K6Of3Wu
2GkUhHJTPteyO7+6Iz2u2suofAIEkg5sc5LFOGOyw96sdaJ4Xv4ve/KMY9GR3705
DK1xQm5h4VMe3h/17RMHHdQHszt+8UncCUOd0qONhXdfAgMBAAGjUzBRMB0GA1Ud
DgQWBBT65ljPbftjSzrHft0ILxpwRAl+9DAfBgNVHSMEGDAWgBT65ljPbftjSzrH
ft0ILxpwRAl+9DAPBgNVHRMBAf8EBTADAQH/MA0GCSqGSIb3DQEBCwUAA4IBAQA8
tWoFwd62W5d9/qLJceXj8e5Jm+v3vNMGmrLMl6F/1OvWNHq0CuXUK0B5SY3tyDgT
WkedfaBPdiWRfMHhM6PkhnGXUuwNak3YbF20IcGtwFWJv3s3A3MY5GZZQ0HDddMu
1isKJ5yF/Egxi8owHOePtxZrihjwhs3Z7Q5i/mT+EfUrWejy52vHcNFg80Pk3t6i
9X8Qq+2+yGHRgWNtYK6112YwFhSQoxGogHPBDZQBhwH2J0z0BvpWqMn2eDuewBSx
ohrbTfrrmDUvKMzPvhLvfUOI7u8nFZL9BXsEhttvhwG1KxQSN3NYCoK2Oted00Xy
0uekmFA0URpysYQfjh8i
-----END CERTIFICATE-----"#;


fn main() {
    env_logger::init();
    
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {
        println!("DriveGuard Updater");
        println!("Usage:");
        println!("  updater.exe --check <manifest_url> <current_version>");
        println!("  updater.exe --download <version> <url> <checksum>");
        println!("  updater.exe --apply <version> <current_version>");
        println!("  updater.exe --rollback");
        return;
    }
    
    match args[1].as_str() {
        "--check" => {
            if args.len() < 4 {
                eprintln!("Error: --check requires manifest URL and current version");
                std::process::exit(1);
            }
            check_for_updates(&args[2], &args[3]);
        }
        "--download" => {
            if args.len() < 5 {
                eprintln!("Error: --download requires version, URL, and checksum");
                std::process::exit(1);
            }
            download_update(&args[2], &args[3], &args[4]);
        }
        "--apply" => {
            if args.len() < 4 {
                eprintln!("Error: --apply requires version and current version");
                std::process::exit(1);
            }
            apply_update(&args[2], &args[3]);
        }
        "--rollback" => {
            rollback_update();
        }
        _ => {
            eprintln!("Error: Unknown command: {}", args[1]);
            std::process::exit(1);
        }
    }
}

fn check_for_updates(manifest_url: &str, current_version: &str) {
    log::info!("Checking for updates from: {}", manifest_url);
    log::info!("Current version: {}", current_version);
    
    // Create HTTP client with embedded CA certificate
    let ca_cert = match reqwest::Certificate::from_pem(CUSTOM_CA_CERT) {
        Ok(cert) => cert,
        Err(e) => {
            log::error!("Failed to parse embedded CA certificate: {}", e);
            // Fallback: accept invalid certs
            log::warn!("Falling back to accepting invalid certificates");
            return check_for_updates_insecure(manifest_url, current_version);
        }
    };
    
    let client = match reqwest::blocking::Client::builder()
        .add_root_certificate(ca_cert)
        .build()
    {
        Ok(c) => c,
        Err(e) => {
            log::error!("Failed to create HTTP client: {}", e);
            std::process::exit(1);
        }
    };
    
    let response = match client.get(manifest_url).send() {
        Ok(resp) => resp,
        Err(e) => {
            log::error!("Failed to fetch manifest: {}", e);
            std::process::exit(1);
        }
    };
    
    let manifest: UpdateManifest = match response.json() {
        Ok(m) => m,
        Err(e) => {
            log::error!("Failed to parse manifest: {}", e);
            std::process::exit(1);
        }
    };
    
    log::info!("Latest version: {}", manifest.latest_version);
    
    let current = match Version::parse(current_version) {
        Ok(v) => v,
        Err(e) => {
            log::error!("Failed to parse current version: {}", e);
            std::process::exit(1);
        }
    };
    
    let latest = match Version::parse(&manifest.latest_version) {
        Ok(v) => v,
        Err(e) => {
            log::error!("Failed to parse latest version: {}", e);
            std::process::exit(1);
        }
    };
    
    if latest > current {
        println!("UPDATE_AVAILABLE:{}", manifest.latest_version);
        
        if let Some(version_info) = manifest.versions.get(&manifest.latest_version) {
            println!("URL:{}", version_info.download_url);
            println!("CHECKSUM:{}", version_info.checksum_sha256);
            println!("SIZE:{}", version_info.file_size_bytes);
            println!("BREAKING:{}", version_info.breaking_changes);
            println!("IS_TEST:{}", latest.is_test());
        }
    } else {
        println!("UP_TO_DATE");
    }
}

fn check_for_updates_insecure(manifest_url: &str, current_version: &str) {
    log::info!("Checking for updates from: {}", manifest_url);
    log::info!("Current version: {}", current_version);
    
    // Fallback: accept any certificate (development only)
    let client = match reqwest::blocking::Client::builder()
        .danger_accept_invalid_certs(true)
        .build()
    {
        Ok(c) => c,
        Err(e) => {
            log::error!("Failed to create HTTP client: {}", e);
            std::process::exit(1);
        }
    };
    
    let response = match client.get(manifest_url).send() {
        Ok(resp) => resp,
        Err(e) => {
            log::error!("Failed to fetch manifest: {}", e);
            std::process::exit(1);
        }
    };
    
    let manifest: UpdateManifest = match response.json() {
        Ok(m) => m,
        Err(e) => {
            log::error!("Failed to parse manifest: {}", e);
            std::process::exit(1);
        }
    };
    
    log::info!("Latest version: {}", manifest.latest_version);
    
    let current = match Version::parse(current_version) {
        Ok(v) => v,
        Err(e) => {
            log::error!("Failed to parse current version: {}", e);
            std::process::exit(1);
        }
    };
    
    let latest = match Version::parse(&manifest.latest_version) {
        Ok(v) => v,
        Err(e) => {
            log::error!("Failed to parse latest version: {}", e);
            std::process::exit(1);
        }
    };
    
    if latest > current {
        println!("UPDATE_AVAILABLE:{}", manifest.latest_version);
        
        if let Some(version_info) = manifest.versions.get(&manifest.latest_version) {
            println!("URL:{}", version_info.download_url);
            println!("CHECKSUM:{}", version_info.checksum_sha256);
            println!("SIZE:{}", version_info.file_size_bytes);
            println!("BREAKING:{}", version_info.breaking_changes);
            println!("IS_TEST:{}", latest.is_test());
        }
    } else {
        println!("UP_TO_DATE");
    }
}

fn download_update(version: &str, url: &str, expected_checksum: &str) {
    log::info!("Downloading update {} from {}", version, url);
    
    let filename = format!("driveguard_v{}.exe", version);
    let download_path = PathBuf::from("updates").join("downloads").join(&filename);
    
    // Create downloads directory
    fs::create_dir_all(download_path.parent().unwrap()).ok();
    
    // Create HTTP client with embedded CA certificate
    let ca_cert = match reqwest::Certificate::from_pem(CUSTOM_CA_CERT) {
        Ok(cert) => cert,
        Err(e) => {
            log::error!("Failed to parse embedded CA certificate: {}", e);
            log::warn!("Falling back to accepting invalid certificates");
            return download_update_insecure(version, url, expected_checksum);
        }
    };
    
    let client = match reqwest::blocking::Client::builder()
        .add_root_certificate(ca_cert)
        .build()
    {
        Ok(c) => c,
        Err(e) => {
            log::error!("Failed to create HTTP client: {}", e);
            std::process::exit(1);
        }
    };
    
    // Download file
    let mut response = match client.get(url).send() {
        Ok(resp) => resp,
        Err(e) => {
            log::error!("Failed to download: {}", e);
            std::process::exit(1);
        }
    };
    
    let mut file = match fs::File::create(&download_path) {
        Ok(f) => f,
        Err(e) => {
            log::error!("Failed to create file: {}", e);
            std::process::exit(1);
        }
    };
    
    if let Err(e) = std::io::copy(&mut response, &mut file) {
        log::error!("Failed to write file: {}", e);
        std::process::exit(1);
    }
    
    log::info!("Downloaded to: {}", download_path.display());
    
    // Verify checksum
    let contents = fs::read(&download_path).unwrap();
    let mut hasher = Sha256::new();
    hasher.update(&contents);
    let checksum = format!("{:x}", hasher.finalize());
    
    if checksum != expected_checksum {
        log::error!("Checksum mismatch! Expected: {}, Got: {}", expected_checksum, checksum);
        fs::remove_file(&download_path).ok();
        std::process::exit(1);
    }
    
    log::info!("Checksum verified successfully");
    println!("DOWNLOAD_COMPLETE:{}", download_path.display());
}

fn download_update_insecure(version: &str, url: &str, expected_checksum: &str) {
    log::info!("Downloading update {} from {} (insecure)", version, url);
    
    let filename = format!("driveguard_v{}.exe", version);
    let download_path = PathBuf::from("updates").join("downloads").join(&filename);
    
    // Create downloads directory
    fs::create_dir_all(download_path.parent().unwrap()).ok();
    
    let client = match reqwest::blocking::Client::builder()
        .danger_accept_invalid_certs(true)
        .build()
    {
        Ok(c) => c,
        Err(e) => {
            log::error!("Failed to create HTTP client: {}", e);
            std::process::exit(1);
        }
    };
    
    // Download file
    let mut response = match client.get(url).send() {
        Ok(resp) => resp,
        Err(e) => {
            log::error!("Failed to download: {}", e);
            std::process::exit(1);
        }
    };
    
    let mut file = match fs::File::create(&download_path) {
        Ok(f) => f,
        Err(e) => {
            log::error!("Failed to create file: {}", e);
            std::process::exit(1);
        }
    };
    
    if let Err(e) = std::io::copy(&mut response, &mut file) {
        log::error!("Failed to write file: {}", e);
        std::process::exit(1);
    }
    
    log::info!("Downloaded to: {}", download_path.display());
    
    // Verify checksum
    let contents = fs::read(&download_path).unwrap();
    let mut hasher = Sha256::new();
    hasher.update(&contents);
    let checksum = format!("{:x}", hasher.finalize());
    
    if checksum != expected_checksum {
        log::error!("Checksum mismatch! Expected: {}, Got: {}", expected_checksum, checksum);
        fs::remove_file(&download_path).ok();
        std::process::exit(1);
    }
    
    log::info!("Checksum verified successfully");
    println!("DOWNLOAD_COMPLETE:{}", download_path.display());
}

fn apply_update(version: &str, current_version: &str) {
    log::info!("Applying update from {} to version {}", current_version, version);
    
    let new_exe = PathBuf::from("updates")
        .join("downloads")
        .join(format!("driveguard_v{}.exe", version));
    
    if !new_exe.exists() {
        log::error!("Update file not found: {}", new_exe.display());
        std::process::exit(1);
    }
    
    let current_exe = PathBuf::from("driveguard.exe");
    
    // Create backup
    let backup_dir = PathBuf::from("updates").join(format!("v{}", current_version));
    fs::create_dir_all(&backup_dir).ok();
    let backup_path = backup_dir.join("driveguard.exe");
    
    log::info!("Backing up current version to: {}", backup_path.display());
    if let Err(e) = fs::copy(&current_exe, &backup_path) {
        log::error!("Failed to create backup: {}", e);
        std::process::exit(1);
    }
    
    // Replace executable
    log::info!("Replacing executable...");
    if let Err(e) = fs::remove_file(&current_exe) {
        log::error!("Failed to remove old executable: {}", e);
        std::process::exit(1);
    }
    
    if let Err(e) = fs::copy(&new_exe, &current_exe) {
        log::error!("Failed to copy new executable: {}", e);
        // Try to restore backup
        fs::copy(&backup_path, &current_exe).ok();
        std::process::exit(1);
    }
    
    log::info!("Update applied successfully!");
    
    // Clean up download
    fs::remove_file(&new_exe).ok();
    
    // Restart DriveGuard
    log::info!("Restarting DriveGuard...");
    Command::new(&current_exe)
        .spawn()
        .expect("Failed to restart DriveGuard");
    
    println!("UPDATE_APPLIED:{}", version);
}

fn rollback_update() {
    log::info!("Rolling back to previous version");
    
    // Find most recent backup
    let updates_dir = PathBuf::from("updates");
    
    let mut versions: Vec<PathBuf> = fs::read_dir(&updates_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_dir())
        .map(|e| e.path())
        .collect();
    
    versions.sort();
    versions.reverse();
    
    if let Some(backup_dir) = versions.first() {
        let backup_exe = backup_dir.join("driveguard.exe");
        
        if backup_exe.exists() {
            let current_exe = PathBuf::from("driveguard.exe");
            fs::copy(&backup_exe, &current_exe).expect("Failed to restore backup");
            
            log::info!("Rolled back to: {}", backup_dir.display());
            println!("ROLLBACK_COMPLETE");
            return;
        }
    }
    
    log::error!("No backup found to rollback to");
    std::process::exit(1);
}