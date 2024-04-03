use crate::config;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;

pub async fn process_log(config: config::DNDTriggerConfig) -> Result<(), Box<dyn std::error::Error>> {
    let mut child = Command::new("log")
        .args(&["stream", "--predicate", "subsystem == 'com.apple.donotdisturb'"])
        .stdout(std::process::Stdio::piped())
        .spawn()?;

    let stdout = child.stdout.take().unwrap();
    let mut reader = BufReader::new(stdout).lines();

    while let Some(line) = reader.next_line().await? {
        if let Some(dnd_state) = parse_line(&line).await {
            if dnd_state {
                if let Some(on_enable) = &config.on_enable {
                    Command::new(on_enable).spawn()?;
                }
            } else {
                if let Some(on_disable) = &config.on_disable {
                    Command::new(on_disable).spawn()?;
                }
            }
        }
    }

    Ok(())
}

/// Parse the log entry, returning True or False if it specifies the DND state.
/// If the log entry does not specify the DND state, return None.
async fn parse_line(log: &str) -> Option<bool> {
    // The entries we are looking for begin with "State was updated: currentState"
    log.contains("State was updated: currentState")
        // If activeModeConfiguration: (null) is present, DND is inactive, otherwise it is active
        .then(|| !log.contains("activeModeConfiguration: (null)"))
}