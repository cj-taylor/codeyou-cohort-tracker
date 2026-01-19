use anyhow::Result;
use serde::Deserialize;

const REPO_OWNER: &str = "cj-taylor";
const REPO_NAME: &str = "codeyou-cohort-tracker";
const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Debug, Deserialize)]
struct Release {
    tag_name: String,
}

pub async fn check_for_updates() -> Result<Option<String>> {
    let client = reqwest::Client::new();
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
