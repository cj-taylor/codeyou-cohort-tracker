use anyhow::Result;
use serde::Deserialize;
use std::fs;
use std::path::PathBuf;
use std::time::SystemTime;

const REPO_OWNER: &str = "cj-taylor";
const REPO_NAME: &str = "codeyou-cohort-tracker";
const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");
const CHECK_INTERVAL_SECONDS: u64 = 86400; // 24 hours

#[derive(Debug, Deserialize)]
struct Release {
    tag_name: String,
}

fn cache_file_path() -> PathBuf {
    dirs::cache_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("cohort-tracker")
        .join("last_update_check")
}

fn should_check_for_updates() -> bool {
    let cache_path = cache_file_path();

    if let Ok(metadata) = fs::metadata(&cache_path) {
        if let Ok(modified) = metadata.modified() {
            if let Ok(duration) = SystemTime::now().duration_since(modified) {
                return duration.as_secs() > CHECK_INTERVAL_SECONDS;
            }
        }
    }

    true // Check if cache doesn't exist or can't be read
}

fn update_check_cache() -> Result<()> {
    let cache_path = cache_file_path();
    if let Some(parent) = cache_path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&cache_path, "")?;
    Ok(())
}

pub async fn check_for_updates() -> Result<Option<String>> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()?;

    let url = format!(
        "https://api.github.com/repos/{}/{}/releases/latest",
        REPO_OWNER, REPO_NAME
    );

    let response = client
        .get(&url)
        .header("User-Agent", "cohort-tracker")
        .send()
        .await?;

    if !response.status().is_success() {
        return Ok(None);
    }

    let release: Release = response.json().await?;

    // Compare versions - if tag is different from current, there's an update
    if release.tag_name != format!("v{}", CURRENT_VERSION) {
        Ok(Some(release.tag_name))
    } else {
        Ok(None)
    }
}

pub async fn check_and_notify() {
    if !should_check_for_updates() {
        return;
    }

    match check_for_updates().await {
        Ok(Some(version)) => {
            eprintln!("\n🔔 A new version is available: {}", version);
            eprintln!("   Run 'cohort-tracker update' to upgrade\n");
            let _ = update_check_cache();
        }
        Ok(None) => {
            let _ = update_check_cache();
        }
        Err(_) => {
            // Silently fail - don't block normal operation
        }
    }
}

pub async fn perform_update() -> Result<()> {
    println!("Checking for updates...");

    let status = self_update::backends::github::Update::configure()
        .repo_owner(REPO_OWNER)
        .repo_name(REPO_NAME)
        .bin_name("cohort-tracker")
        .current_version(CURRENT_VERSION)
        .build()?
        .update()?;

    if status.updated() {
        println!("✓ Updated to version {}", status.version());
    } else {
        println!("Already up to date (version {})", CURRENT_VERSION);
    }

    Ok(())
}
