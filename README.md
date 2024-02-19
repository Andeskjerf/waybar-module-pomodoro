# How to use

* compile & install binary
* add to ~/.config/waybar/config

```json
    "custom/pomodoro": {
        "format": "{} {icon}",
				"return-type": "json",
				"format-icons": {
					"running": "󰔟",
					"stopped": "󱦠",
				},
        "exec": "path-to-binary",
        "on-click": "path-to-binary toggle",
    },
```
