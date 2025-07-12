use anyhow::{anyhow, Result};
use std::fs;
use std::path::{Path, PathBuf};
use tokio::process::Command as TokioCommand;
use tracing::{info, warn};

const AST_GREP_VERSION: &str = "0.38.7";
const GITHUB_RELEASE_URL: &str = "https://github.com/ast-grep/ast-grep/releases/download";

pub struct BinaryManager {
    binary_dir: PathBuf,
    binary_path: PathBuf,
}

impl BinaryManager {
    pub fn new() -> Result<Self> {
        let binary_dir = Self::get_binary_dir()?;
        let binary_path = binary_dir.join(Self::get_binary_name());

        Ok(Self {
            binary_dir,
            binary_path,
        })
    }

    pub async fn ensure_binary(&self) -> Result<PathBuf> {
        if self.binary_exists() {
            info!("ast-grep binary found at: {}", self.binary_path.display());
            return Ok(self.binary_path.clone());
        }

        // Try to find system binary first
        if let Ok(system_path) = self.find_system_binary().await {
            info!("Using system ast-grep binary at: {}", system_path.display());
            return Ok(system_path);
        }

        // Download and bundle the binary
        info!("Downloading ast-grep binary...");
        self.download_binary().await?;

        Ok(self.binary_path.clone())
    }

    pub fn get_binary_path(&self) -> Result<String> {
        if self.binary_exists() {
            return Ok(self.binary_path.to_string_lossy().to_string());
        }

        // Try to find system binary
        if let Ok(output) = std::process::Command::new("which").arg("ast-grep").output() {
            if output.status.success() {
                return Ok(String::from_utf8_lossy(&output.stdout).trim().to_string());
            }
        }

        // Check common system locations
        let common_paths = [
            "/usr/local/bin/ast-grep",
            "/usr/bin/ast-grep",
            "/opt/homebrew/bin/ast-grep",
            "/usr/local/cargo/bin/ast-grep",
            "ast-grep", // Fallback to PATH
        ];

        for path in &common_paths {
            if Path::new(path).exists() || path == &"ast-grep" {
                return Ok(path.to_string());
            }
        }

        // Return bundled path even if it doesn't exist yet
        Ok(self.binary_path.to_string_lossy().to_string())
    }

    async fn find_system_binary(&self) -> Result<PathBuf> {
        let output = TokioCommand::new("which").arg("ast-grep").output().await?;

        if output.status.success() {
            let path_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
            return Ok(PathBuf::from(path_str));
        }

        Err(anyhow!("System binary not found"))
    }

    fn binary_exists(&self) -> bool {
        self.binary_path.exists() && self.binary_path.is_file()
    }

    async fn download_binary(&self) -> Result<()> {
        let download_url = self.get_download_url()?;

        info!("Downloading from: {}", download_url);

        // Create binary directory
        fs::create_dir_all(&self.binary_dir)?;

        // Download the archive
        let client = reqwest::Client::new();
        let response = client.get(&download_url).send().await?;

        if !response.status().is_success() {
            return Err(anyhow!(
                "Failed to download binary: HTTP {}",
                response.status()
            ));
        }

        let archive_bytes = response.bytes().await?;

        // Extract based on platform
        if cfg!(windows) {
            self.extract_zip(&archive_bytes).await?;
        } else {
            // Try zip first, then tar.gz
            match self.extract_zip(&archive_bytes).await {
                Ok(_) => {}
                Err(_) => {
                    warn!("ZIP extraction failed, trying tar.gz extraction");
                    self.extract_tar_gz(&archive_bytes).await?;
                }
            }
        }

        // Make binary executable on Unix systems
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&self.binary_path)?.permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&self.binary_path, perms)?;
        }

        info!(
            "Binary downloaded and extracted to: {}",
            self.binary_path.display()
        );
        Ok(())
    }

    async fn extract_zip(&self, archive_bytes: &[u8]) -> Result<()> {
        use std::io::Cursor;
        let cursor = Cursor::new(archive_bytes);
        let mut archive = zip::ZipArchive::new(cursor)?;

        for i in 0..archive.len() {
            let mut file = archive.by_index(i)?;
            let file_name = file.name();

            // Look for the binary file
            if file_name.ends_with("ast-grep") || file_name.ends_with("ast-grep.exe") {
                let mut buffer = Vec::new();
                std::io::copy(&mut file, &mut buffer)?;
                fs::write(&self.binary_path, buffer)?;
                return Ok(());
            }

            // Handle nested directories
            if file_name.contains("ast-grep")
                && (file_name.ends_with("/ast-grep")
                    || file_name.ends_with("\\ast-grep")
                    || file_name.ends_with("/ast-grep.exe")
                    || file_name.ends_with("\\ast-grep.exe"))
            {
                let mut buffer = Vec::new();
                std::io::copy(&mut file, &mut buffer)?;
                fs::write(&self.binary_path, buffer)?;
                return Ok(());
            }
        }

        Err(anyhow!("ast-grep binary not found in ZIP archive"))
    }

    async fn extract_tar_gz(&self, archive_bytes: &[u8]) -> Result<()> {
        use flate2::read::GzDecoder;
        use std::io::Cursor;

        let cursor = Cursor::new(archive_bytes);
        let decoder = GzDecoder::new(cursor);
        let mut archive = tar::Archive::new(decoder);

        for entry in archive.entries()? {
            let mut entry = entry?;
            let path = entry.path()?;
            let file_name = path.to_string_lossy();

            if file_name.ends_with("ast-grep") || file_name.ends_with("ast-grep.exe") {
                let mut buffer = Vec::new();
                std::io::copy(&mut entry, &mut buffer)?;
                fs::write(&self.binary_path, buffer)?;
                return Ok(());
            }
        }

        Err(anyhow!("ast-grep binary not found in tar.gz archive"))
    }

    fn get_download_url(&self) -> Result<String> {
        let platform_suffix = self.get_platform_suffix()?;
        Ok(format!(
            "{GITHUB_RELEASE_URL}/{AST_GREP_VERSION}/app-{platform_suffix}.zip"
        ))
    }

    fn get_platform_suffix(&self) -> Result<&'static str> {
        let suffix = match (std::env::consts::OS, std::env::consts::ARCH) {
            ("linux", "x86_64") => "x86_64-unknown-linux-gnu",
            ("linux", "aarch64") => "aarch64-unknown-linux-gnu",
            ("macos", "x86_64") => "x86_64-apple-darwin",
            ("macos", "aarch64") => "aarch64-apple-darwin",
            ("windows", "x86_64") => "x86_64-pc-windows-msvc",
            ("windows", "x86") => "i686-pc-windows-msvc",
            (os, arch) => return Err(anyhow!("Unsupported platform: {} {}", os, arch)),
        };
        Ok(suffix)
    }

    fn get_binary_dir() -> Result<PathBuf> {
        // Use a directory relative to the executable
        let exe_path = std::env::current_exe()?;
        let exe_dir = exe_path
            .parent()
            .ok_or_else(|| anyhow!("Cannot determine executable directory"))?;

        let binary_dir = exe_dir.join("bundled_binaries");
        Ok(binary_dir)
    }

    fn get_binary_name() -> &'static str {
        if cfg!(windows) {
            "ast-grep.exe"
        } else {
            "ast-grep"
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_platform_suffix() {
        let manager = BinaryManager::new().unwrap();
        let suffix = manager.get_platform_suffix();
        assert!(suffix.is_ok());
    }

    #[test]
    fn test_get_binary_dir() {
        let dir = BinaryManager::get_binary_dir();
        assert!(dir.is_ok());
    }
}
