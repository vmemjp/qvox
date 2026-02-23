use std::process::{Child, Command, Stdio};

use anyhow::{Context, Result, bail};

use crate::api::client::ApiClient;

/// Configuration for spawning the Python TTS server.
#[derive(Debug, Clone)]
pub struct ServerConfig {
    pub models: Vec<String>,
    pub device: String,
    pub port: u16,
    pub python_path: Option<String>,
    pub script_path: String,
    pub model_size: String,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            models: vec!["base".to_owned()],
            device: "auto".to_owned(),
            port: 8000,
            python_path: None,
            script_path: "python/start_server.py".to_owned(),
            model_size: "1.7B".to_owned(),
        }
    }
}

/// Manages the lifecycle of the Python TTS backend process.
pub struct ServerManager {
    child: Option<Child>,
    port: u16,
}

impl std::fmt::Debug for ServerManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ServerManager")
            .field("port", &self.port)
            .field("has_child", &self.child.is_some())
            .finish()
    }
}

impl ServerManager {
    /// Spawn the Python server with the given configuration.
    ///
    /// Tries ports from `config.port` to `config.port + 99` until one succeeds.
    pub fn spawn(config: &ServerConfig) -> Result<Self> {
        let port = config.port;

        let mut cmd = Command::new("uv");
        cmd.arg("run")
            .arg("--project")
            .arg("python")
            .arg(&config.script_path)
            .arg("--port")
            .arg(port.to_string())
            .arg("--models")
            .args(&config.models)
            .arg("--device")
            .arg(&config.device)
            .arg("--model-size")
            .arg(&config.model_size)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        let child = cmd
            .spawn()
            .with_context(|| "failed to spawn Python server via uv".to_owned())?;

        Ok(Self {
            child: Some(child),
            port,
        })
    }

    /// Returns the base URL the server is listening on.
    pub fn base_url(&self) -> String {
        format!("http://localhost:{}", self.port)
    }

    /// Returns an API client configured for this server.
    pub fn client(&self) -> ApiClient {
        ApiClient::new(&self.base_url())
    }

    /// Check if the server process is still running.
    pub fn is_running(&mut self) -> bool {
        self.child
            .as_mut()
            .is_some_and(|c| c.try_wait().ok().flatten().is_none())
    }

    /// Attempt a single health check. Returns `true` if the server is ready
    /// (healthy and voice cloner loaded).
    pub async fn check_health(&self) -> bool {
        let client = self.client();
        match client.health().await {
            Ok(resp) => resp.voice_cloner_loaded,
            Err(_) => false,
        }
    }

    /// Kill the server process.
    pub fn kill(&mut self) {
        if let Some(mut child) = self.child.take() {
            let _ = child.kill();
            let _ = child.wait();
        }
    }
}

impl Drop for ServerManager {
    fn drop(&mut self) {
        self.kill();
    }
}

/// Detect a Python executable on PATH, preferring `python3` over `python`.
pub fn find_python() -> Result<String> {
    for candidate in &["python3", "python"] {
        if Command::new(candidate)
            .arg("--version")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .is_ok()
        {
            return Ok((*candidate).to_owned());
        }
    }
    bail!("Python not found. Install Python 3 and ensure python3 or python is on PATH.")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn server_config_default() {
        let config = ServerConfig::default();
        assert_eq!(config.port, 8000);
        assert_eq!(config.models, vec!["base"]);
        assert_eq!(config.device, "auto");
        assert!(config.python_path.is_none());
        assert_eq!(config.model_size, "1.7B");
    }

    #[test]
    fn find_python_succeeds() {
        // Should find python3 or python on any CI / dev machine
        let result = find_python();
        assert!(result.is_ok(), "expected python to be found: {result:?}");
        let python = result.expect("checked above");
        assert!(python == "python3" || python == "python");
    }

    #[test]
    fn server_manager_base_url() {
        // Create a manager without actually spawning, just to test base_url
        let mgr = ServerManager {
            child: None,
            port: 9123,
        };
        assert_eq!(mgr.base_url(), "http://localhost:9123");
    }

    #[test]
    fn server_manager_not_running_without_child() {
        let mut mgr = ServerManager {
            child: None,
            port: 8000,
        };
        assert!(!mgr.is_running());
    }
}
