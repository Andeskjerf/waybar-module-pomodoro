use crate::{LONG_BREAK_TIME, SHORT_BREAK_TIME, WORK_TIME, MINUTE};

pub struct Config {
    pub work_time: u16,
    pub short_break: u16,
    pub long_break: u16,
    pub no_icons: bool,
}

impl Config {
    pub fn from_options(options: Vec<String>) -> Self {
        // define & initialize the times with the default values
        // need to be mut since we might change them based on user arguments
        let mut work_time: u16 = WORK_TIME;
        let mut short_break: u16 = SHORT_BREAK_TIME;
        let mut long_break: u16 = LONG_BREAK_TIME;
        let mut no_icons = false;

        options.iter().for_each(|opt| match opt.as_str() {
            "-w" | "--work" => work_time = get_config_value(&options, vec!["-w", "--work"]) * MINUTE,
            "-s" | "--shortbreak" => {
                short_break = get_config_value(&options, vec!["-s", "--shortbreak"]) * MINUTE
            }
            "-l" | "--longbreak" => {
                long_break = get_config_value(&options, vec!["-l", "--longbreak"]) * MINUTE
            }
            "--no-icons" => no_icons = true,
            _ => (),
        });

        Self {
            work_time,
            short_break,
            long_break,
            no_icons,
        }
    }
}

fn get_config_value(options: &[String], keys: Vec<&str>) -> u16 {
    let index = options
        .iter()
        .position(|x| keys.contains(&x.as_str()))
        .expect("option specified but no value followed");

    options[index + 1]
        .parse::<u16>()
        .expect("value is not a number")
}
