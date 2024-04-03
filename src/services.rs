use crate::config;

use service_manager::*;
use std::env;
use std::path::PathBuf;
use lazy_static::lazy_static;

lazy_static! {
    static ref CONFIG: config::DNDTriggerConfig = config::read_config();
    static ref LABEL: ServiceLabel = "sh.collin.dndtrigger".parse().unwrap();
    static ref MANAGER: LaunchdServiceManager = LaunchdServiceManager::system();
    static ref BIN_PATH: PathBuf = env::current_exe().expect("Failed to get current executable path");
}

pub fn start_service() {
    println!("Start the service...");
    MANAGER.start(ServiceStartCtx {
        label: LABEL.clone()
    }).expect("Failed to start service");
    println!("Service started");
}

pub fn stop_service() {
    println!("Stopping the service...");
    MANAGER.stop(ServiceStopCtx {
        label: LABEL.clone()
    }).expect("Failed to stop service");
    println!("Service stopped");
}

pub fn install_service() {
    println!("Installing the service...");
    MANAGER.install(ServiceInstallCtx {
        label: LABEL.clone(),
        program: BIN_PATH.clone(),
        args: vec!["run".parse().unwrap()],
        contents: None,
        username: CONFIG.user.clone(),
        working_directory: None,
        environment: None,
    }).expect("Failed to install service");
    println!("Service installed, starting...");
    MANAGER.start(ServiceStartCtx {
        label: LABEL.clone()
    }).expect("Failed to start service");
    println!("Service started");
}

pub fn uninstall_service() {
    println!("Uninstalling the service...");
    MANAGER.uninstall(ServiceUninstallCtx {
        label: LABEL.clone()
    }).expect("Failed to uninstall service");
    println!("Service uninstalled");
}
