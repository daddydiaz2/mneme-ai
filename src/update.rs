/// Update check — query crates.io for latest version.
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const CACHE_DURATION: u64 = 86400; // 24 hours

#[derive(Debug, Clone, serde::Deserialize)]
struct CratesIoResponse {
    #[serde(rename = "crate")]
    crate_info: CrateInfo,
}

#[derive(Debug, Clone, serde::Deserialize)]
struct CrateInfo {
    max_version: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct VersionCache {
    versions: std::collections::HashMap<String, CachedVersion>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct CachedVersion {
    latest: String,
    checked_at: u64,
}

/// Check for updates for a specific crate
pub fn check_update(crate_name: &str, current_version: &str) -> Option<String> {
    // Check cache first
    let cache = load_cache();
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    if let Some(cached) = cache.versions.get(crate_name) {
        if now - cached.checked_at < CACHE_DURATION {
            return compare_versions(current_version, &cached.latest);
        }
    }

    // Query crates.io API
    let url = format!("https://crates.io/api/v1/crates/{}", crate_name);
    if let Ok(response) = ureq::get(&url).call() {
        if let Ok(body) = response.into_body().read_to_string() {
            if let Ok(data) = serde_json::from_str::<CratesIoResponse>(&body) {
                let latest = data.crate_info.max_version;
                // Update cache
                let mut cache = load_cache();
                cache.versions.insert(
                    crate_name.to_string(),
                    CachedVersion {
                        latest: latest.clone(),
                        checked_at: now,
                    },
                );
                save_cache(&cache);
                return compare_versions(current_version, &latest);
            }
        }
    }
    None
}

/// Compare two semver versions
fn compare_versions(current: &str, latest: &str) -> Option<String> {
    if current == latest {
        return None;
    }

    let current_parts: Vec<u32> = current.split('.').filter_map(|p| p.parse().ok()).collect();
    let latest_parts: Vec<u32> = latest.split('.').filter_map(|p| p.parse().ok()).collect();

    for i in 0..3 {
        let c = current_parts.get(i).copied().unwrap_or(0);
        let l = latest_parts.get(i).copied().unwrap_or(0);
        if l > c {
            return Some(format!("v{} → v{} (update available)", current, latest));
        }
        if c > l {
            return None;
        }
    }
    None
}

fn cache_path() -> std::path::PathBuf {
    let base = dirs::config_dir().unwrap_or_else(|| std::path::PathBuf::from("."));
    base.join("mneme-ai").join("update-cache.json")
}

fn load_cache() -> VersionCache {
    let path = cache_path();
    if path.exists() {
        std::fs::read_to_string(&path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or(VersionCache {
                versions: std::collections::HashMap::new(),
            })
    } else {
        VersionCache {
            versions: std::collections::HashMap::new(),
        }
    }
}

fn save_cache(cache: &VersionCache) {
    if let Some(parent) = cache_path().parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    if let Ok(content) = serde_json::to_string(cache) {
        let _ = std::fs::write(cache_path(), content);
    }
}

/// Print update status
pub fn print_update_status(crate_name: &str, current_version: &str) {
    if let Some(msg) = check_update(crate_name, current_version) {
        println!("⚠  {}", msg);
        println!("   Run: cargo install {} --force", crate_name);
    }
}
