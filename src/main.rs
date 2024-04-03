mod config;
mod services;
mod logs;

use tokio::runtime::Runtime;
use clap::{Arg, Command};

fn main() {
    let matches = Command::new("dndtrigger")
        .version("0.1.0")
        .author("Collin Gray <me@collin.sh>")
        .about("Triggers specified actions when Do Not Disturb is toggled on or off.")
        .subcommand(Command::new("start")
            .about("Starts the service"))
        .subcommand(Command::new("stop")
            .about("Stops the service"))
        .subcommand(Command::new("install")
            .about("Installs and starts the service"))
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
        .after_help(
            "The config plist file is located at /Library/Preferences/sh.collin.dndtrigger.\
            plist, although there is no need to edit it manually\n\
            The service is installed as 'sh.collin.dndtrigger' and can be managed directly with \
            `launchctl`.\n\
            Note that because this binary modifies Launch Daemons, most commands will have to be \
            run with root access."
        )
        .get_matches();

    match matches.subcommand() {
        Some(("start", _)) => {
            services::stop_service();
        }
        Some(("stop", _)) => {
            services::start_service();
        }
        Some(("install", _)) => {
            services::install_service();
        }
        Some(("uninstall", _)) => {
            services::uninstall_service();
        }
        Some(("restart", _)) => {
            services::stop_service();
            services::start_service();
        }
        Some(("config", config_matches)) => {
            config::write_config(
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
            let config = config::read_config();
            let rt = Runtime::new().unwrap();
            rt.block_on(logs::process_log(config)).unwrap();
        }
        _ => {}
    }
}
