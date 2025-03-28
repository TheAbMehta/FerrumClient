use std::path::PathBuf;
use std::process::Stdio;
use std::time::Duration;
use thiserror::Error;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, Command};
use tokio::time::timeout;

#[derive(Debug, Error)]
pub enum SubprocessError {
    #[error("Failed to spawn process: {0}")]
    SpawnFailed(#[from] std::io::Error),
    
    #[error("Process crashed during startup with exit code: {0:?}")]
    ProcessCrashed(Option<i32>),
    
    #[error("Startup timeout: Done message not received within {0:?}")]
    StartupTimeout(Duration),
    
    #[error("Failed to send stop command: {0}")]
    StopCommandFailed(std::io::Error),
    
    #[error("Process is not running")]
    NotRunning,
}

pub struct PumpkinServer {
    binary_path: PathBuf,
    child: Option<Child>,
}

impl PumpkinServer {
    pub fn new(binary_path: PathBuf) -> Self {
        Self {
            binary_path,
            child: None,
        }
    }

    pub async fn start(&mut self) -> Result<(), SubprocessError> {
        let mut cmd = Command::new(&self.binary_path);
        cmd.stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit());

        #[cfg(unix)]
        {
            unsafe {
                cmd.pre_exec(|| {
                    libc::setpgid(0, 0);
                    Ok(())
                });
            }
        }

        let mut child = cmd.spawn()?;
        
        let stdout = child.stdout.take().expect("stdout should be piped");
        let mut reader = BufReader::new(stdout);
        
        let startup_timeout = Duration::from_secs(30);
        let result = timeout(startup_timeout, async {
            let mut line = String::new();
            loop {
                line.clear();
                match reader.read_line(&mut line).await {
                    Ok(0) => {
                        let exit_status = child.wait().await.ok();
                        return Err(SubprocessError::ProcessCrashed(
                            exit_status.and_then(|s| s.code())
                        ));
                    }
                    Ok(_) => {
                        if line.contains("Done") && line.contains("s)!") {
                            break;
                        }
                    }
                    Err(e) => return Err(SubprocessError::SpawnFailed(e)),
                }
            }
            Ok(())
        }).await;

        match result {
            Ok(Ok(())) => {
                child.stdout = Some(reader.into_inner());
                self.child = Some(child);
                Ok(())
            }
            Ok(Err(e)) => {
                let _ = child.kill().await;
                Err(e)
            }
            Err(_) => {
                let _ = child.kill().await;
                Err(SubprocessError::StartupTimeout(startup_timeout))
            }
        }
    }

    pub fn is_running(&self) -> bool {
        self.child.is_some()
    }

    pub async fn stop(&mut self) -> Result<(), SubprocessError> {
        let child = self.child.as_mut().ok_or(SubprocessError::NotRunning)?;
        
        if let Some(mut stdin) = child.stdin.take() {
            stdin.write_all(b"stop\n").await
                .map_err(SubprocessError::StopCommandFailed)?;
            drop(stdin);
        }
        
        let graceful_timeout = Duration::from_secs(30);
        let result = timeout(graceful_timeout, child.wait()).await;
        
        match result {
            Ok(Ok(_)) => {
                self.child = None;
                Ok(())
            }
            Ok(Err(e)) => Err(SubprocessError::SpawnFailed(e)),
            Err(_) => {
                child.kill().await?;
                self.child = None;
                Ok(())
            }
        }
    }

    pub async fn kill(&mut self) -> Result<(), SubprocessError> {
        let child = self.child.as_mut().ok_or(SubprocessError::NotRunning)?;
        child.kill().await?;
        self.child = None;
        Ok(())
    }
}

impl Drop for PumpkinServer {
    fn drop(&mut self) {
        if let Some(child) = self.child.take() {
            let _ = std::process::Command::new("kill")
                .arg("-9")
                .arg(child.id().unwrap().to_string())
                .spawn();
        }
    }
}
