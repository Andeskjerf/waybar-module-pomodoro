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
