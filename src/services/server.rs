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
        consts::{MINUTE, SLEEP_DURATION},
    },
};

use super::{
    cache,
    timer::{CycleType, Timer},
};

pub fn send_notification(cycle_type: CycleType) {
    match Notification::new()
        .summary("Pomodoro")
        .body(match cycle_type {
            CycleType::Work => "Time to work!",
            CycleType::ShortBreak => "Time for a short break!",
            CycleType::LongBreak => "Time for a long break!",
        })
        .show()
    {
        Ok(_) => {}
        Err(_) => panic!("Failed to send notification"),
    };
}

fn format_time(elapsed_time: u16, max_time: u16) -> String {
    let time = max_time - elapsed_time;
    let minute = time / MINUTE;
    let second = time % MINUTE;
    format!("{:02}:{:02}", minute, second)
}

fn print_message(value: String, tooltip: &str, class: &str) {
    println!(
        "{{\"text\": \"{}\", \"tooltip\": \"{}\", \"class\": \"{}\", \"alt\": \"{}\"}}",
        value, tooltip, class, class
    );
}

fn process_message(state: &mut Timer, message: &str) {
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
            process_message(&mut state, &message);
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
        print_message(
            utils::helper::trim_whitespace(&format!("{} {} {}", value_prefix, value, cycle_icon)),
            tooltip.as_str(),
            &class,
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
