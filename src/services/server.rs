use std::{
    env, fs,
    io::{Error, Read, Write},
    os::unix::net::{UnixListener, UnixStream},
    path::Path,
    sync::mpsc::{Receiver, Sender},
    thread,
};

use notify_rust::Notification;

use crate::{
    models::{config::Config, message::Message},
    utils::{
        self,
        consts::{HOUR, MINUTE, SLEEP_DURATION},
    },
};

use super::{
    cache,
    timer::{CycleType, Timer},
};

pub fn send_notification(cycle_type: CycleType) {
    if let Err(e) = Notification::new()
        .summary("Pomodoro")
        .body(match cycle_type {
            CycleType::Work => "Time to work!",
            CycleType::ShortBreak => "Time for a short break!",
            CycleType::LongBreak => "Time for a long break!",
        })
        .show()
    {
        println!("err: send_notification, err == {e}");
    }
}

fn format_time(elapsed_time: u16, max_time: u16) -> String {
    let time = max_time - elapsed_time;

    let hour = time / HOUR;
    let minute = (time % HOUR) / MINUTE;
    let second = time % MINUTE;

    if hour > 0 {
        return format!("{:02}:{:02}:{:02}", hour, minute, second);
    }

    format!("{:02}:{:02}", minute, second)
}

fn create_message(value: String, tooltip: &str, class: &[String]) -> String {
    let class_json = format!(
        "[{}]",
        class
            .iter()
            .map(|c| format!("\"{}\"", c))
            .collect::<Vec<String>>()
            .join(",")
    );

    format!(
        "{{\"text\": \"{}\", \"tooltip\": \"{}\", \"class\": {}, \"alt\": \"{}\"}}",
        value, tooltip, class_json, ""
    )
}

fn process_message(state: &mut Timer, message: &str, config: &Config) {
    if let Ok(msg) = Message::decode(message) {
        match msg.name() {
            "set-work" => state.set_time(CycleType::Work, msg.value() as u16),
            "set-short" => state.set_time(CycleType::ShortBreak, msg.value() as u16),
            "set-long" => state.set_time(CycleType::LongBreak, msg.value() as u16),
            _ => println!("err: invalid command, {}", msg.name()),
        }
    } else {
        match message {
            "start" => {
                state.running = true;
            }
            "stop" => {
                state.running = false;
            }
            "toggle" => {
                state.running = !state.running;
            }
            "reset" => {
                state.reset();
            }
            "skip" => {
                state.skip(&config);
            }
            _ => {
                println!("Unknown message: {}", message);
            }
        }
    }
}

fn handle_client(rx: Receiver<String>, socket_path: String, config: Config) {
    let socket_nr = socket_path
        .chars()
        .filter_map(|c| c.to_digit(10))
        .fold(0, |acc, digit| acc * 10 + digit) as i32;

    let mut state = Timer::new(
        config.work_time,
        config.short_break,
        config.long_break,
        socket_nr,
    );

    if config.persist {
        let _ = cache::restore(&mut state, &config);
    }

    loop {
        if let Ok(message) = rx.try_recv() {
            process_message(&mut state, &message, &config);
        }

        let value = format_time(state.elapsed_time, state.get_current_time());
        let value_prefix = config.get_play_pause_icon(state.running);
        let tooltip = format!(
            "{} pomodoro{} completed this session",
            state.session_completed,
            if state.session_completed > 1 || state.session_completed == 0 {
                "s"
            } else {
                ""
            }
        );
        let class = state.get_class();
        let cycle_icon = config.get_cycle_icon(state.is_break());
        state.update_state(&config);
        println!(
            "{}",
            create_message(
                utils::helper::trim_whitespace(&format!(
                    "{} {} {}",
                    value_prefix, value, cycle_icon
                )),
                tooltip.as_str(),
                &class,
            )
        );

        if state.running {
            state.increment_time();
        }

        if config.persist {
            let _ = cache::store(&state);
        }

        std::thread::sleep(SLEEP_DURATION);
    }
}

fn delete_socket(socket_path: &str) {
    if Path::new(&socket_path).exists() {
        fs::remove_file(socket_path).unwrap();
    }
}

pub fn spawn_server(socket_path: &str, config: Config) {
    delete_socket(socket_path);

    let listener = UnixListener::bind(socket_path).unwrap();
    let (tx, rx): (Sender<String>, Receiver<String>) = std::sync::mpsc::channel();
    {
        let socket_path = socket_path.to_owned();
        thread::spawn(|| handle_client(rx, socket_path, config));
    }

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                // read incoming data
                let mut message = String::new();
                stream
                    .read_to_string(&mut message)
                    .expect("Failed to read UNIX stream");

                if message.contains("exit") {
                    delete_socket(socket_path);
                    break;
                }
                tx.send(message.to_string()).unwrap();
            }
            Err(err) => println!("Error: {}", err),
        }
    }
}

pub fn get_existing_sockets(binary_name: &str) -> Vec<String> {
    let mut files: Vec<String> = vec![];

    if let Ok(paths) = env::temp_dir().read_dir() {
        for path in paths {
            let name = path.unwrap().path().to_str().unwrap().to_string();
            if name.contains(binary_name) {
                files.push(name);
            }
        }
    }

    files
}

pub fn send_message_socket(socket_path: &str, msg: &str) -> Result<(), Error> {
    let mut stream = UnixStream::connect(socket_path)?;
    stream.write_all(msg.as_bytes())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::models::config;
    use crate::LONG_BREAK_TIME;
    use crate::SHORT_BREAK_TIME;
    use fs::File;
    use utils::consts::WORK_TIME;

    use super::*;
    use crate::services::server::CycleType;

    fn create_timer() -> Timer {
        Timer::new(WORK_TIME, SHORT_BREAK_TIME, LONG_BREAK_TIME, 0)
    }

    fn get_time(timer: &Timer, cycle: CycleType) -> u16 {
        match cycle {
            CycleType::Work => timer.times[0],
            CycleType::ShortBreak => timer.times[1],
            CycleType::LongBreak => timer.times[2],
        }
    }

    #[test]
    fn test_send_notification_work() {
        send_notification(CycleType::Work);
    }

    #[test]
    fn test_send_notification_short_break() {
        send_notification(CycleType::ShortBreak);
    }

    #[test]
    fn test_send_notification_long_break() {
        send_notification(CycleType::LongBreak);
    }

    #[test]
    fn test_format_time() {
        assert_eq!(format_time(300, 600), "05:00");
        assert_eq!(format_time(59, 60), "00:01");
        assert_eq!(format_time(0, 120), "02:00");
    }

    #[test]
    fn test_create_message() {
        let message = "Pomodoro";
        let tooltip = "Tooltip";
        let class = vec!["Class".to_owned()];

        let result = create_message(message.to_string(), tooltip, &class);
        let expected = format!(
            "{{\"text\": \"{}\", \"tooltip\": \"{}\", \"class\": [\"{}\"], \"alt\": \"{}\"}}",
            message,
            tooltip,
            // FIXME: yeah
            class.first().unwrap(),
            ""
        );
        assert!(result == expected);
    }

    #[test]
    fn test_process_message_set_work() {
        let mut timer = create_timer();
        process_message(
            &mut timer,
            &Message::new("set-work", 30).encode(),
            &Config::default(),
        );
        assert_eq!(get_time(&timer, CycleType::Work), 30 * MINUTE);
    }

    #[test]
    fn test_process_message_set_short() {
        let mut timer = create_timer();
        process_message(
            &mut timer,
            &Message::new("set-short", 3).encode(),
            &Config::default(),
        );
        assert_eq!(get_time(&timer, CycleType::ShortBreak), 3 * MINUTE);
    }

    #[test]
    fn test_process_message_set_long() {
        let mut timer = create_timer();
        process_message(
            &mut timer,
            &Message::new("set-long", 10).encode(),
            &Config::default(),
        );
        assert_eq!(get_time(&timer, CycleType::LongBreak), 10 * MINUTE);
    }

    #[test]
    fn test_process_message_start() {
        let mut timer = create_timer();
        process_message(&mut timer, "start", &Config::default());
        assert!(timer.running);
    }

    #[test]
    fn test_process_message_stop() {
        let mut timer = create_timer();
        process_message(&mut timer, "stop", &Config::default());
        assert!(!timer.running);
    }

    #[test]
    fn test_process_message_skip() {
        let mut timer = create_timer();
        timer.current_index = timer.times.len() - 1;
        let config = Config::default();
        timer.iterations = config.intervals;
        process_message(&mut timer, "skip", &config);
        assert!((timer.session_completed == 1));
    }

    // TODO:
    // #[tokio::test]
    // async fn test_spawn_server() {
    // }

    // TODO:
    // #[tokio::test]
    // async fn test_handle_client() {
    // }

    // TODO:
    // #[tokio::test]
    // async fn test_send_message_socket() {
    // }

    #[test]
    fn test_delete_socket() {
        let socket_path = "/tmp/waybar-module-pomodoro_test_socket";
        std::fs::File::create(socket_path).unwrap();
        assert!(std::path::Path::new(socket_path).exists());

        delete_socket(socket_path);
        assert!(!std::path::Path::new(socket_path).exists());
    }

    #[test]
    fn test_get_existing_sockets() {
        let binary_name = "waybar-module-pomodoro_test";
        let temp_dir = env::temp_dir();
        let socket_path = temp_dir.join(binary_name);

        File::create(&socket_path).unwrap();

        let result = get_existing_sockets(binary_name);
        assert!(result.contains(&socket_path.to_string_lossy().to_string()));

        std::fs::remove_file(socket_path).unwrap();
    }
}
