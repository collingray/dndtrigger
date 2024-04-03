use crate::config;

use service_manager::*;
use std::env;
use std::path::PathBuf;
use std::process::Command;
use std::io::{self, Write};
use lazy_static::lazy_static;

static LABEL: &str = "sh.collin.dndtrigger";

lazy_static! {
    static ref CONFIG: config::DNDTriggerConfig = config::read_config();
    static ref SERVICE_LABEL: ServiceLabel = LABEL.parse().unwrap();
    static ref MANAGER: LaunchdServiceManager = LaunchdServiceManager::system();
    static ref BIN_PATH: PathBuf = env::current_exe().expect("Failed to get current executable path");
}

pub fn start_service() {
    println!("Start the service...");
    MANAGER.start(ServiceStartCtx {
        label: SERVICE_LABEL.clone()
    }).expect("Failed to start service");
    println!("Service started");
}

pub fn stop_service() {
    println!("Stopping the service...");
    MANAGER.stop(ServiceStopCtx {
        label: SERVICE_LABEL.clone()
    }).expect("Failed to stop service");
    println!("Service stopped");
}

pub fn install_service() {
    println!("Installing the service...");
    MANAGER.install(ServiceInstallCtx {
        label: SERVICE_LABEL.clone(),
        program: BIN_PATH.clone(),
        args: vec!["run".parse().unwrap()],
        contents: None,
        username: CONFIG.user.clone(),
        working_directory: None,
        environment: None,
    }).expect("Failed to install service");
    println!("Service installed, starting...");
    MANAGER.start(ServiceStartCtx {
        label: SERVICE_LABEL.clone()
    }).expect("Failed to start service");
    println!("Service started");
}

pub fn uninstall_service() {
    println!("Uninstalling the service...");
    MANAGER.uninstall(ServiceUninstallCtx {
        label: SERVICE_LABEL.clone()
    }).expect("Failed to uninstall service");
    println!("Service uninstalled");
}

// The service_manager crate does not have a binding for fetching the service status, so we
// implement it directly using launchctl
pub fn get_service_status() {
    // We have to capture and restream the output here instead of using spawn in order to ensure
    // that the stream closes
    let output = Command::new("launchctl")
        .args(&["print", &format!("system/{}", LABEL)])
        .output()
        .expect("Failed to get service status");

    io::stdout().write_all(&output.stdout).expect("Failed to write to stdout");
    io::stderr().write_all(&output.stderr).expect("Failed to write to stderr");
}