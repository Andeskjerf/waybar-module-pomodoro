use crate::{
    BREAK_ICON, LONG_BREAK_TIME, MINUTE, PAUSE_ICON, PLAY_ICON, SHORT_BREAK_TIME, WORK_ICON,
    WORK_TIME,
};

pub struct Config {
    pub work_time: u16,
    pub short_break: u16,
    pub long_break: u16,
    pub no_icons: bool,
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
        // define & initialize the times with the default values
        // need to be mut since we might change them based on user arguments
        let mut work_time: u16 = WORK_TIME;
        let mut short_break: u16 = SHORT_BREAK_TIME;
        let mut long_break: u16 = LONG_BREAK_TIME;
        let mut no_icons = false;
        let mut play_icon = PLAY_ICON.to_string();
        let mut pause_icon = PAUSE_ICON.to_string();
        let mut work_icon = WORK_ICON.to_string();
        let mut break_icon = BREAK_ICON.to_string();
        let mut autow = false;
        let mut autob = false;

        let binary_path = options.first().unwrap();
        let binary_name = binary_path.split('/').last().unwrap().to_string();

        options.iter().for_each(|opt| match opt.as_str() {
            "-w" | "--work" => {
                work_time = get_config_value(&options, vec!["-w", "--work"])
                    .parse::<u16>()
                    .expect("value is not a number")
                    * MINUTE
            }
            "-s" | "--shortbreak" => {
                short_break = get_config_value(&options, vec!["-s", "--shortbreak"])
                    .parse::<u16>()
                    .expect("value is not a number")
                    * MINUTE
            }
            "-l" | "--longbreak" => {
                long_break = get_config_value(&options, vec!["-l", "--longbreak"])
                    .parse::<u16>()
                    .expect("value is not a number")
                    * MINUTE
            }
            "-p" | "--play" => play_icon = get_config_value(&options, vec!["-p", "--play"]),
            "-a" | "--pause" => pause_icon = get_config_value(&options, vec!["-a", "--pause"]),
            "-o" | "--work-icon" => {
                work_icon = get_config_value(&options, vec!["-o", "--work-icon"])
            }
            "-b" | "--break-icon" => {
                break_icon = get_config_value(&options, vec!["-b", "--break-icon"])
            }
            "--autow" => autow = true,
            "--autob" => autob = true,
            "--no-icons" => no_icons = true,
            _ => (),
        });

        Self {
            work_time,
            short_break,
            long_break,
            no_icons,
            play_icon,
            pause_icon,
            work_icon,
            break_icon,
            autow,
            autob,
            binary_name,
        }
    }
}

fn get_config_value(options: &[String], keys: Vec<&str>) -> String {
    let index = options
        .iter()
        .position(|x| keys.contains(&x.as_str()))
        .expect("option specified but no value followed");

    options[index + 1].to_owned()
}
