# What is this?

A pomodoro timer for your system bar, intended for Waybar!

I have not tested other status bars, but hopefully it should also work for others. Waybar accepts a json with various data to display info for a module. If your bar also accepts json, then maybe it'll work the same as it does on Waybar.

It follows the same rules as pomodoro: 4 cycles of work and short breaks, followed by a long break.

# How to use

* compile & install binary
```bash
git clone git@github.com:Andeskjerf/waybar-module-pomodoro.git
cd waybar-module-pomodoro
cargo build --release
```
You can find the binary in `workingDir/target/release/waybar-module-pomodoro`.

Place the file in a folder you have in your $PATH. E.g `/home/user/.local/bin`
  
* add to `~/.config/waybar/config`

```json
"custom/pomodoro": {
	"format": "{} {icon}",
	"return-type": "json",
	"format-icons": {
		"work": "󰔟",
		"break": "",
	},
	"exec": "waybar-module-pomodoro",
	"on-click": "waybar-module-pomodoro toggle",
	"on-click-right": "waybar-module-pomodoro reset",
},
```

Include the module in your bar and you're set!

You can check how many pomodoros you've completed this session by hovering the module and checking its tooltip.

# Options / arguments?

```
usage: waybar-module-pomodoro [options] [operation]
    options:
        -h, --help                  Prints this help message
        -w, --work <value>          Sets how long a work cycle is, in minutes. default: 25
        -s, --shortbreak <value>    Sets how long a short break is, in minutes. default: 5
        -l, --longbreak <value>     Sets how long a long break is, in minutes. default: 15
        --no-icons                  Disable the pause/play icon
    operations:
        toggle                      Toggles the timer
        start                       Start the timer
        pause                       Pause the timer
        reset                       Reset timer to initial state
```
