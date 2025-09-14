use async_trait::async_trait;
use std::error::Error;
use std::process::Stdio;
use std::time::Duration;
use tokio::process::Command as TokioCommand;
use tokio::time;

/// Abstraction for command execution to enable testing without real commands
#[async_trait]
pub trait CommandExecutor {
    async fn execute(&self, command: &str, args: &[&str]) -> Result<String, Box<dyn Error>>;
    async fn execute_with_timeout(
        &self,
        command: &str,
        args: &[&str],
        timeout_duration: Duration,
    ) -> Result<String, Box<dyn Error>>;
}

/// Real command executor using std::process::Command
pub struct RealCommandExecutor;

#[async_trait]
impl CommandExecutor for RealCommandExecutor {
    async fn execute(&self, command: &str, args: &[&str]) -> Result<String, Box<dyn Error>> {
        let output = TokioCommand::new(command)
            .args(args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await?;

        if output.status.success() {
            Ok(String::from_utf8(output.stdout)?)
        } else {
            let stderr = String::from_utf8(output.stderr)?;
            Err(format!("Command failed: {}", stderr).into())
        }
    }

    async fn execute_with_timeout(
        &self,
        command: &str,
        args: &[&str],
        timeout_duration: Duration,
    ) -> Result<String, Box<dyn Error>> {
        let result = time::timeout(timeout_duration, self.execute(command, args)).await;
        match result {
            Ok(output) => output,
            Err(_) => Err(format!("Command timed out after {:?}", timeout_duration).into()),
        }
    }
}

/// Demo command executor that returns predefined responses
pub struct DemoCommandExecutor;

impl DemoCommandExecutor {
    fn get_demo_response(&self, command: &str, args: &[&str]) -> Option<&'static str> {
        match (command, args) {
            ("zpool", ["list", "-H", "-o", "name"]) => Some("boot-pool\ndata\nusb-backup\n"),
            ("zpool", ["status"]) => Some(include_str!("../demo/zpool_status.txt")),
            ("zpool", ["iostat", "-v"]) => Some(include_str!("../demo/zpool_iostat.txt")),
            ("arcstat", ["-f", "hit%,miss%,read,arcsz,c", "1", "1"]) => {
                Some("100.0 0.0 1247 49720066048 49910562816\n")
            }
            ("arcstat", ["1", "1"]) => Some("100.0 0.0 1247 49720066048 49910562816\n"),
            ("echo", ["|", "arcstat"]) => Some("100.0 0.0 1247 49720066048 49910562816\n"),
            _ => None,
        }
    }
}

#[async_trait]
impl CommandExecutor for DemoCommandExecutor {
    async fn execute(&self, command: &str, args: &[&str]) -> Result<String, Box<dyn Error>> {
        if let Some(response) = self.get_demo_response(command, args) {
            Ok(response.to_string())
        } else {
            Err(format!("Demo: Command not mocked: {} {:?}", command, args).into())
        }
    }

    async fn execute_with_timeout(
        &self,
        command: &str,
        args: &[&str],
        _timeout: Duration,
    ) -> Result<String, Box<dyn Error>> {
        self.execute(command, args).await
    }
}
