# dndtrigger
A simple cli/service for MacOS to execute scripts when Do Not Disturb is toggled on or off.

## Usage
```bash
# Configure a script to run when Do Not Disturb is enabled
dndtrigger config --on_enable ~/do_something.sh

# Configure a script to run when Do Not Disturb is disabled
dndtrigger config --on_disable ~/do_something_else.sh

# Optionally configure the service to run as a user (default is root)
dndtrigger config --user <username>

# Install and enable the service
dndtrigger enable 

# Get the current status of the service
dndtrigger status

# Restart the service
dndtrigger restart

# Disable and uninstall the service
dndtrigger disable

# Run the service in the foreground (not recommended outside of testing purposes)
dndtrigger run
```

## License
This project is licensed under the terms of the MIT license. See [LICENSE](LICENSE) for additional details.
