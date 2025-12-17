use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tokio::sync::broadcast;
use tracing::{error, info, warn};

use crate::config::CardanoConfig;
use crate::models::OuraEvent;

/// Service for managing the Oura subprocess and reading blockchain events
pub struct OuraReader {
    config: CardanoConfig,
}

impl OuraReader {
    // Create a new OuraReader with the given Configuration
    pub fn new(config: CardanoConfig) -> Self {
        Self { config }
    }

    // Start reading evetnts from the Oura and send then throught the channel
    pub async fn start(
        &self,
        tx: broadcast::Sender<OuraEvent>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        info!("Starting Oura dump command...");
        info!("Network: {}", self.config.network_name);
        info!("Connecting to: {}", self.config.relay);
        info!("This may take a moment to connect to the Cardano Node...");

        // Spawn oura dump command with proper flags to only output JSON
        let mut cmd = Command::new("oura");
        cmd.arg("dump")
            .arg(self.config.relay)
            .arg("--bearer")
            .arg("tcp");

        if let Some(magic) = self.config.magic {
            cmd.arg("--magic").arg(magic.to_string());
        }

        let mut child = cmd
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            // spawn starts the process asynchronously
            // Returns a Child process handle (child) that can be used to read output or wait for the process to finish.
            // -? propagates errors if starting the process fails.
            .spawn()?;

        // It takes the piped output to the terminal to the stdout and if it fails it panics with the message
        let stdout = child.stdout.take().expect("Failed to capture stdout");
        let stderr = child.stderr.take().expect("Failed to capture stderr");

        // Spawn task to log stderr
        tokio::spawn(async move {
            let reader = BufReader::new(stderr);
            let mut lines = reader.lines();
            while let Ok(Some(line)) = lines.next_line().await {
                error!("oura stderr: {}", line);
            }
        });

        let reader = BufReader::new(stdout);
        let mut lines = reader.lines();

        while let Ok(Some(line)) = lines.next_line().await {
            // Skip empty lines and non-JSON lines
            if line.trim().is_empty() || !line.trim().starts_with('{') {
                continue;
            }

            // Parse Json Line
            match serde_json::from_str::<OuraEvent>(&line) {
                Ok(oura_event) => {
                    // Send to channel for processing
                    if let Err(e) = tx.send(oura_event) {
                        // Channel is likely full or closed (no receivers)
                        // This is normal when no WebSocket clients are connected
                        warn!("Failed to send oura event (channel full/closed): {}", e);
                    }
                }
                Err(e) => {
                    warn!(
                        "Failed to parse JSON: {} - Line: {}",
                        e,
                        &line[..line.len().min(100)]
                    );
                }
            }
        }

        // Wait for child process
        let status = child.wait().await?;
        error!("Oura process exited with status: {}", status);

        Ok(())
    }
}
