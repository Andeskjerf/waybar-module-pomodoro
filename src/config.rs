use crate::{
    models::message::Message, BREAK_ICON, LONG_BREAK_TIME, MINUTE, PAUSE_ICON, PLAY_ICON,
    SHORT_BREAK_TIME, WORK_ICON, WORK_TIME,
};

pub const OPERATIONS: [&str; 4] = ["toggle", "start", "stop", "reset"];
pub const SET_OPERATIONS: [&str; 3] = ["set-work", "set-short", "set-long"];

pub struct Config {
    pub work_time: u16,
    pub short_break: u16,
    pub long_break: u16,
    pub no_icons: bool,
    pub no_work_icons: bool,
    pub play_icon: String,
    pub pause_icon: String,
    pub work_icon: String,
    pub break_icon: String,
    pub autow: bool,
    pub autob: bool,
    pub binary_name: String,
}

impl Config {
    pub fn from_options(options: Vec<String>) -> Self {
        let mut work_time: u16 = WORK_TIME;
        let mut short_break: u16 = SHORT_BREAK_TIME;
        let mut long_break: u16 = LONG_BREAK_TIME;
        let mut no_icons = false;
        let mut no_work_icons = false;
        let mut play_icon = PLAY_ICON.to_string();
        let mut pause_icon = PAUSE_ICON.to_string();
        let mut work_icon = WORK_ICON.to_string();
        let mut break_icon = BREAK_ICON.to_string();
        let mut autow = false;
        let mut autob = false;

        let binary_path = options.first().unwrap();
        let binary_name = binary_path.split('/').last().unwrap().to_string();

        for opt in options.iter() {
            let val = get_config_value(&options, vec![opt]);
            if val.is_none() {
                continue;
            }

            let val = val.unwrap().clone();
            match opt.as_str() {
                "-w" | "--work" => {
                    work_time = val.parse::<u16>().expect("value is not a number") * MINUTE
                }
                "-s" | "--shortbreak" => {
                    short_break = val.parse::<u16>().expect("value is not a number") * MINUTE
                }
                "-l" | "--longbreak" => {
                    long_break = val.parse::<u16>().expect("value is not a number") * MINUTE
                }
                "-p" | "--play" => play_icon = val,
                "-a" | "--pause" => pause_icon = val,
                "-o" | "--work-icon" => work_icon = val,
                "-b" | "--break-icon" => break_icon = val,
                "--autow" => autow = true,
                "--autob" => autob = true,
                "--no-icons" => no_icons = true,
                "--no-work-icons" => no_work_icons = true,
                _ => (),
            }
        }

        Self {
            work_time,
            short_break,
            long_break,
            no_icons,
            no_work_icons,
            play_icon,
            pause_icon,
            work_icon,
            break_icon,
            autow,
            autob,
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
