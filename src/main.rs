use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command as Cmd;
use tokio::runtime::Runtime;
use clap::{Arg, Command};
use service_manager::*;
use once_cell::sync::Lazy;
use std::env;
use plist;
use serde::{Deserialize, Serialize};

async fn stream_logs_and_process(config: DNDTriggerConfig) -> Result<(), Box<dyn std::error::Error>> {
    let mut child = Cmd::new("log")
        .args(&["stream", "--predicate", "subsystem == 'com.apple.donotdisturb'"])
        .stdout(std::process::Stdio::piped())
        .spawn()?;

    let stdout = child.stdout.take().unwrap();
    let mut reader = BufReader::new(stdout).lines();

    while let Some(line) = reader.next_line().await? {
        if let Some(dnd_state) = parse_log_entry(&line).await {
            if dnd_state {
                if let Some(on_enable) = &config.on_enable {
                    Cmd::new(on_enable).spawn()?;
                }
            } else {
                if let Some(on_disable) = &config.on_disable {
                    Cmd::new(on_disable).spawn()?;
                }
            }
        }
    }

    Ok(())
}


/// Parse the log entry, returning True or False if it specifies the DND state.
/// If the log entry does not specify the DND state, return None.
async fn parse_log_entry(log: &str) -> Option<bool> {
    // The entries we are looking for begin with "State was updated: currentState"
    log.contains("State was updated: currentState")
        // If activeModeConfiguration: (null) is present, DND is inactive, otherwise it is active
        .then(|| !log.contains("activeModeConfiguration: (null)"))
}

#[derive(Default)]
#[derive(Deserialize, Serialize)]
struct DNDTriggerConfig {
    on_enable: Option<String>,
    on_disable: Option<String>,
    user: Option<String>,
}

static CONFIG_PATH: &str = "/Library/Preferences/sh.collin.dndtrigger.plist";

fn write_config(on_enable: Option<&str>, on_disable: Option<&str>, user: Option<&str>) {
    // Write the configuration options to a file
    let old_config: DNDTriggerConfig = plist::from_file(&CONFIG_PATH).unwrap_or_default();

    let new_config = DNDTriggerConfig {
        on_enable: on_enable.map(|s| s.to_string()).or_else(|| old_config.on_enable),
        on_disable: on_disable.map(|s| s.to_string()).or_else(|| old_config.on_disable),
        user: user.map(|s| s.to_string()).or_else(|| old_config.user).filter(|s| s != "root"),
    };

    plist::to_file_xml(&CONFIG_PATH, &new_config).expect("Failed to write config");
}

fn load_config() -> DNDTriggerConfig {
    // Load the configuration options from a file
    plist::from_file(&CONFIG_PATH).unwrap_or_default()
}

fn main() {
    let matches = Command::new("dndtrigger")
        .version("1.0")
        .author("Collin Gray <me@collin.sh>")
        .about("Triggers specified actions when Do Not Disturb is toggled on or off.")
        .subcommand(Command::new("enable")
            .about("Enables the service"))
        .subcommand(Command::new("disable")
            .about("Disables the service"))
        .subcommand(Command::new("install")
            .about("Installs the service"))
        .subcommand(Command::new("uninstall")
            .about("Uninstalls the service"))
        .subcommand(Command::new("restart")
            .about("Restarts the service"))
        .subcommand(Command::new("run")
            .about("Runs the listener in the foreground. (Not recommended, use 'install' to run as a service)"))
        .subcommand(Command::new("config")
            .about("Configures the actions to be run")
            .arg(Arg::new("on_enable")
                .help("The path to the binary/script to run when DND is enabled")
                .required(false)
                .long("on_enable"))
            .arg(Arg::new("on_disable")
                .help("The path to the binary/script to run when DND is disabled")
                .required(false)
                .long("on_disable"))
            .arg(Arg::new("user")
                .help("The user to run the service as. Set to 'root' by default.")
                .required(false)
                .long("user"))
            .arg_required_else_help(true))
        .arg_required_else_help(true)
        .get_matches();

    let config = load_config();

    let service_label: ServiceLabel = "sh.collin.dndtrigger".parse().unwrap();
    let manager = Lazy::new(|| {
        let mut manager = LaunchdServiceManager::system();
        manager.config.install.keep_alive = true;
        manager
    });
    let binary_path = env::current_exe().expect("Failed to get current executable path");

    match matches.subcommand() {
        Some(("enable", _)) => {
            println!("Enabling the service...");
            manager.start(ServiceStartCtx {
                label: service_label.clone()
            }).expect("Failed to start service");
            println!("Service enabled");
        }
        Some(("disable", _)) => {
            println!("Disabling the service...");
            manager.stop(ServiceStopCtx {
                label: service_label.clone()
            }).expect("Failed to stop service");
            println!("Service disabled");
        }
        Some(("install", _)) => {
            println!("Installing the service...");
            manager.install(ServiceInstallCtx {
                label: service_label.clone(),
                program: binary_path,
                args: vec!["run".parse().unwrap()],
                contents: None,
                username: config.user,
                working_directory: None,
                environment: None,
            }).expect("Failed to install service");
            println!("Service installed, enabling...");
            manager.start(ServiceStartCtx {
                label: service_label.clone()
            }).expect("Failed to start service");
            println!("Service enabled");
        }
        Some(("uninstall", _)) => {
            println!("Uninstalling the service...");
            manager.uninstall(ServiceUninstallCtx {
                label: service_label.clone()
            }).expect("Failed to uninstall service");
            println!("Service uninstalled");
        }
        Some(("restart", _)) => {
            println!("Restarting the service...");
            manager.stop(ServiceStopCtx {
                label: service_label.clone()
            }).and_then(|_| {
                manager.start(ServiceStartCtx {
                    label: service_label.clone()
                })
            }).expect("Failed to restart service");
            println!("Service restarted");
        }
        Some(("config", config_matches)) => {
            write_config(
                config_matches.get_one::<String>("on_enable").map(|s| s.as_str()),
                config_matches.get_one::<String>("on_disable").map(|s| s.as_str()),
                config_matches.get_one::<String>("user").map(|s| s.as_str())
            );
            println!(
                "Config has been updated.\n\
                If the Daemon is currently running, it will need to be restarted for changes \
                to take effect.\n\
                This can be done by running `dndtrigger restart`.\n\
                If you changed the user, you will need to reinstall the service."
            );
        }
        Some(("run", _)) => {
            let rt = Runtime::new().unwrap();
            rt.block_on(stream_logs_and_process(config)).unwrap();
        }
        _ => {}
    }
}
