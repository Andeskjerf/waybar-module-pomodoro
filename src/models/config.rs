use crate::{
    models::message::Message, utils::consts::MAX_ITERATIONS, BREAK_ICON, LONG_BREAK_TIME, MINUTE,
    PAUSE_ICON, PLAY_ICON, SHORT_BREAK_TIME, WORK_ICON, WORK_TIME,
};

pub const OPERATIONS: [&str; 4] = ["toggle", "start", "stop", "reset"];
pub const SET_OPERATIONS: [&str; 3] = ["set-work", "set-short", "set-long"];

pub struct Config {
    pub work_time: u16,
    pub short_break: u16,
    pub long_break: u16,
    pub intervals: u8,
    pub no_icons: bool,
    pub no_work_icons: bool,
    pub play_icon: String,
    pub pause_icon: String,
    pub work_icon: String,
    pub break_icon: String,
    pub autow: bool,
    pub autob: bool,
    pub persist: bool,
    pub binary_name: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            work_time: Default::default(),
            short_break: Default::default(),
            long_break: Default::default(),
            intervals: MAX_ITERATIONS,
            no_icons: Default::default(),
            no_work_icons: Default::default(),
            play_icon: PLAY_ICON.to_string(),
            pause_icon: PAUSE_ICON.to_string(),
            work_icon: WORK_ICON.to_string(),
            break_icon: BREAK_ICON.to_string(),
            autow: Default::default(),
            autob: Default::default(),
            persist: Default::default(),
            binary_name: Default::default(),
        }
    }
}

impl Config {
    pub fn from_options(options: Vec<String>) -> Self {
        let mut work_time: u16 = WORK_TIME;
        let mut short_break: u16 = SHORT_BREAK_TIME;
        let mut long_break: u16 = LONG_BREAK_TIME;
        let mut intervals: u8 = MAX_ITERATIONS;
        let mut no_icons = false;
        let mut no_work_icons = false;
        let mut play_icon = PLAY_ICON.to_string();
        let mut pause_icon = PAUSE_ICON.to_string();
        let mut work_icon = WORK_ICON.to_string();
        let mut break_icon = BREAK_ICON.to_string();
        let mut autow = false;
        let mut autob = false;
        let mut persist = false;

        let binary_path = options.first().unwrap();
        let binary_name = binary_path.split('/').next_back().unwrap().to_string();

        for opt in options.iter() {
            match opt.as_str() {
                "-w" | "--work" => {
                    let unparsed = get_config_value_except(&options, opt);
                    match unparsed.parse::<u16>() {
                        Ok(val) => work_time = val * MINUTE,
                        Err(_) => println!("err: invalid value for {opt}. val == {unparsed}"),
                    }
                }
                "-s" | "--shortbreak" => {
                    let unparsed = get_config_value_except(&options, opt);
                    match unparsed.parse::<u16>() {
                        Ok(val) => short_break = val * MINUTE,
                        Err(_) => println!("err: invalid value for {opt}. val == {unparsed}"),
                    }
                }
                "-l" | "--longbreak" => {
                    let unparsed = get_config_value_except(&options, opt);
                    match unparsed.parse::<u16>() {
                        Ok(val) => long_break = val * MINUTE,
                        Err(_) => println!("err: invalid value for {opt}. val == {unparsed}"),
                    }
                }
                "-i" | "--intervals" => {
                    let unparsed = get_config_value_except(&options, opt);
                    match unparsed.parse::<u8>() {
                        Ok(val) => intervals = val,
                        Err(_) => println!("err: invalid value for {opt}. val == {unparsed}"),
                    }
                }
                "-p" | "--play" => play_icon = get_config_value_except(&options, opt).clone(),
                "-a" | "--pause" => pause_icon = get_config_value_except(&options, opt),
                "-o" | "--work-icon" => work_icon = get_config_value_except(&options, opt),
                "-b" | "--break-icon" => break_icon = get_config_value_except(&options, opt),
                "--autow" => autow = true,
                "--autob" => autob = true,
                "--persist" => persist = true,
                "--no-icons" => no_icons = true,
                "--no-work-icons" => no_work_icons = true,
                _ => (),
            }
        }

        Self {
            work_time,
            short_break,
            long_break,
            intervals,
            no_icons,
            no_work_icons,
            play_icon,
            pause_icon,
            work_icon,
            break_icon,
            autow,
            autob,
            persist,
            binary_name,
        }
    }

    pub fn get_play_pause_icon(&self, running: bool) -> &str {
        if self.no_icons {
            return "";
        }

        if !running {
            &self.play_icon
        } else {
            &self.pause_icon
        }
    }

    pub fn get_cycle_icon(&self, is_break: bool) -> &str {
        if self.no_work_icons {
            return "";
        }

        if !is_break {
            &self.work_icon
        } else {
            &self.break_icon
        }
    }
}

fn get_config_value_except(options: &[String], opt: &str) -> String {
    get_config_value(options, vec![opt])
        .unwrap_or_else(|| panic!("err: {opt} specified but no value was provided"))
        .clone()
}

pub fn get_config_value<'a>(options: &'a [String], keys: Vec<&'a str>) -> Option<&'a String> {
    match options.iter().position(|x| keys.contains(&x.as_str())) {
        Some(index) => options.get(index + 1).to_owned(),
        None => None,
    }
}

pub fn parse_set_operations(args: Vec<String>) -> Vec<Message> {
    let mut set_operation: Vec<Message> = vec![];
    for elem in SET_OPERATIONS {
        if !args.contains(&elem.to_string()) {
            continue;
        }

        let val = get_config_value(&args, vec![elem]);
        if val.is_none() {
            continue;
        }

        let val = val.unwrap();
        if let Ok(val) = val.parse::<i32>() {
            if val > 0 {
                set_operation.push(Message::new(elem, val));
            } else {
                println!("{elem}: value must be higher than 0, ignoring");
            }
        }
    }
    set_operation
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_from_options_default() {
        let options = vec!["waybar-module-pomodoro_test".to_string()];
        let config = Config::from_options(options);

        assert_eq!(config.work_time, WORK_TIME);
        assert_eq!(config.short_break, SHORT_BREAK_TIME);
        assert_eq!(config.long_break, LONG_BREAK_TIME);
        assert!(!config.no_icons);
        assert!(!config.no_work_icons);
        assert_eq!(config.play_icon, PLAY_ICON.to_string());
        assert_eq!(config.pause_icon, PAUSE_ICON.to_string());
        assert_eq!(config.work_icon, WORK_ICON.to_string());
        assert_eq!(config.break_icon, BREAK_ICON.to_string());
        assert!(!config.autow);
        assert!(!config.autob);
        assert!(!config.persist);
        assert_eq!(config.binary_name, "waybar-module-pomodoro_test");
    }

    #[test]
    fn test_config_from_options_with_custom_values() {
        let options = vec![
            "-w".to_string(),
            "25".to_string(),
            "-s".to_string(),
            "5".to_string(),
            "-l".to_string(),
            "15".to_string(),
            "-i".to_string(),
            "10".to_string(),
            "--play".to_string(),
            "▶️".to_string(),
            "--pause".to_string(),
            "⏸️".to_string(),
            "--work-icon".to_string(),
            "💻".to_string(),
            "--break-icon".to_string(),
            "☕️".to_string(),
            "--autow".to_string(),
            "--persist".to_string(),
        ];
        let config = Config::from_options(options);

        assert_eq!(config.work_time, 25 * MINUTE);
        assert_eq!(config.short_break, 5 * MINUTE);
        assert_eq!(config.long_break, 15 * MINUTE);
        assert_eq!(config.intervals, 10);
        assert!(!config.no_icons);
        assert!(!config.no_work_icons);
        assert_eq!(config.play_icon, "▶️".to_string());
        assert_eq!(config.pause_icon, "⏸️".to_string());
        assert_eq!(config.work_icon, "💻".to_string());
        assert_eq!(config.break_icon, "☕️".to_string());
        assert!(config.autow);
        assert!(!config.autob);
        assert!(config.persist);
    }

    #[test]
    fn test_config_from_options_missing_values() {
        let options = vec![
            "-w".to_string(),
            "--shortbreak".to_string(),
            "5".to_string(),
            "--longbreak".to_string(),
            "15".to_string(),
        ];
        let config = Config::from_options(options);

        assert_eq!(config.work_time, WORK_TIME);
        assert_eq!(config.short_break, 5 * MINUTE);
        assert_eq!(config.long_break, 15 * MINUTE);
    }

    #[test]
    fn test_config_from_options_invalid_values() {
        let options = vec![
            "-w".to_string(),
            "abc".to_string(),
            "--shortbreak".to_string(),
            "def".to_string(),
            "--longbreak".to_string(),
            "ghi".to_string(),
        ];
        let config = Config::from_options(options);

        assert_eq!(config.work_time, WORK_TIME);
        assert_eq!(config.short_break, SHORT_BREAK_TIME);
        assert_eq!(config.long_break, LONG_BREAK_TIME);
    }

    #[test]
    fn test_config_from_options_no_icons() {
        let options = vec!["--no-icons".to_string()];
        let config = Config::from_options(options);

        assert!(config.no_icons);
    }

    #[test]
    fn test_get_play_pause_icon_running() {
        let config = Config::default();
        let icon = config.get_play_pause_icon(true);

        assert_eq!(icon, PAUSE_ICON);
    }

    #[test]
    fn test_get_play_pause_icon_not_running() {
        let config = Config::default();
        let icon = config.get_play_pause_icon(false);

        assert_eq!(icon, PLAY_ICON);
    }

    #[test]
    fn test_get_play_pause_icon_no_icons() {
        let config = Config {
            no_icons: true,
            ..Default::default()
        };
        let icon = config.get_play_pause_icon(true);

        assert_eq!(icon, "");
    }

    #[test]
    fn test_parse_set_operations_valid_values() {
        let args = vec![
            "set-work".to_string(),
            "10".to_string(),
            "set-short".to_string(),
            "5".to_string(),
        ];
        let operations = parse_set_operations(args);

        assert_eq!(operations.len(), 2);
        assert_eq!(operations[0], Message::new("set-work", 10));
        assert_eq!(operations[1], Message::new("set-short", 5));
    }

    #[test]
    fn test_parse_set_operations_invalid_values() {
        let args = vec![
            "work".to_string(),
            "-10".to_string(),
            "break".to_string(),
            "-5".to_string(),
        ];
        let operations = parse_set_operations(args);

        assert_eq!(operations.len(), 0);
    }

    #[test]
    fn test_parse_set_operations_missing_values() {
        let args = vec!["set-work".to_string()];
        let operations = parse_set_operations(args);

        assert_eq!(operations.len(), 0);
    }
}
