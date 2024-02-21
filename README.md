# What is this?

A pomodoro timer for your system bar, intended for Waybar!

I have not tested other status bars, but hopefully it should also work for others. Waybar accepts a json with various data to display info for a module. If your bar also accepts json, then maybe it'll work the same as it does on Waybar.

Currently, the time is hardcoded. A work cycle is 25 minutes, short break is 5 minutes, and a long break is 15 minutes.

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
  
* add to ~/.config/waybar/config

```json
"custom/pomodoro": {
	"format": "{} {icon}",
	"return-type": "json",
	"format-icons": {
		"work": "󰔟",
		"break": "",
	},
	"exec": "path-to-binary",
	"on-click": "path-to-binary toggle",
	"on-click-right": "path-to-binary reset",
},
```

# Arguments?

## `start` & `stop`

`start` starts the timer, and `stop` pauses it.

## `toggle`

`toggle` will start the timer if paused, and pause it if it's started.

## `reset`

`reset` resets the timer completely, resetting it to its initial starting state.
