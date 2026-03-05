use anyhow::Result;

use bb_core::config::Config;

pub async fn run() -> Result<()> {
    let path = Config::config_path();
    if path.exists() {
        std::fs::remove_file(&path)?;
        println!("✓ Logged out. Credentials removed.");
    } else {
        println!("Not logged in.");
    }
    Ok(())
}
