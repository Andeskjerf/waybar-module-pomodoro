use models::{
    config::{parse_set_operations, Config, OPERATIONS},
    message::Message,
};
use notify_rust::Notification;
use services::{
    cache,
    timer::{CycleType, Timer},
};
use signal_hook::{
    consts::{SIGHUP, SIGINT, SIGTERM},
    iterator::Signals,
};
use std::{
    env, fs,
    io::{Error, Read, Write},
    os::unix::net::{UnixListener, UnixStream},
    path::Path,
    sync::mpsc::{Receiver, Sender},
    thread,
};
use utils::consts::{
    BREAK_ICON, LONG_BREAK_TIME, MINUTE, PAUSE_ICON, PLAY_ICON, SHORT_BREAK_TIME, SLEEP_DURATION,
    WORK_ICON, WORK_TIME,
};

mod models;
mod services;
mod utils;

fn send_notification(cycle_type: CycleType) {
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

fn get_class(state: &Timer) -> String {
    // timer hasn't been started yet
    if state.elapsed_millis == 0
        && state.elapsed_time == 0
        && state.iterations == 0
        && state.session_completed == 0
    {
        "".to_owned()
    }
    // timer has been paused
    else if !state.running {
        "pause".to_owned()
    }
    // currently doing some work
    else if !state.is_break() {
        "work".to_owned()
    }
    // currently a break
    else if state.is_break() {
        "break".to_owned()
    } else {
        panic!("invalid condition occurred while setting class!");
    }
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
        let class = &get_class(&state);
        let cycle_icon = config.get_cycle_icon(state.is_break());
        state.update_state(&config);
        print_message(
            utils::helper::trim_whitespace(&format!("{} {} {}", value_prefix, value, cycle_icon)),
            tooltip.as_str(),
            class,
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

fn spawn_server(socket_path: &str, config: Config) {
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

fn get_existing_sockets(binary_name: &str) -> Vec<String> {
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

fn send_message_socket(socket_path: &str, msg: &str) -> Result<(), Error> {
    let mut stream = UnixStream::connect(socket_path)?;
    stream.write_all(msg.as_bytes())?;
    Ok(())
}

// we need to handle signals to ensure a graceful exit
// this is important because we need to remove the sockets on exit
fn process_signals(socket_path: String) {
    // all possible realtime UNIX signals
    let sigrt = 34..64;

    // intentionally ignore realtime signals
    // if we don't do this, the process will terminate if the user sends SIGRTMIN+N to the bar
    let _dont_handle = Signals::new(sigrt.collect::<Vec<i32>>()).unwrap();

    let mut signals = Signals::new([SIGINT, SIGTERM, SIGHUP]).unwrap();
    thread::spawn(move || {
        for _ in signals.forever() {
            send_message_socket(&socket_path, "exit").expect("unable to send message to server");
        }
    });
}

fn main() -> std::io::Result<()> {
    let options = env::args().collect::<Vec<String>>();
    if options.contains(&"--help".to_string()) || options.contains(&"-h".to_string()) {
        print_help();
        return Ok(());
    }

    if options.contains(&"--version".to_string()) || options.contains(&"-v".to_string()) {
        let version: &str = env!("CARGO_PKG_VERSION");
        println!("Ver: {}", version);
        return Ok(());
    }

    let config = Config::from_options(options);

    let mut sockets = get_existing_sockets(&config.binary_name);
    let socket_path: String = format!(
        "{}/{}{}.socket",
        env::temp_dir().display(),
        config.binary_name,
        sockets.len(),
    );

    let operation = env::args()
        .filter(|x| OPERATIONS.contains(&x.as_str()))
        .collect::<Vec<String>>();

    let set_operation = parse_set_operations(env::args().collect::<Vec<String>>());

    if operation.is_empty() && set_operation.is_empty() {
        sockets.push(socket_path.clone());
        process_signals(socket_path.clone());
        spawn_server(&socket_path, config);
        return Ok(());
    }

    for socket in sockets {
        if !operation.is_empty() {
            match send_message_socket(&socket, &operation[0]) {
                Ok(_) => {}
                Err(_) => println!("warn: failed to connect to {}", socket),
            };
        }
        for msg in &set_operation {
            match send_message_socket(&socket, &msg.encode()) {
                Ok(_) => {}
                Err(_) => println!("warn: failed to connect to {}", socket),
            };
        }
    }
    Ok(())
}

fn print_help() {
    println!(
        r#"usage: waybar-module-pomodoro [options] [operation]
    options:
        -h, --help                  Prints this help message
        -v, --version               Prints the version string
        -w, --work <value>          Sets how long a work cycle is, in minutes. default: {}
        -s, --shortbreak <value>    Sets how long a short break is, in minutes. default: {}
        -l, --longbreak <value>     Sets how long a long break is, in minutes. default: {}

        -p, --play <value>          Sets custom play icon/text. default: {}
        -a, --pause <value>         Sets custom pause icon/text. default: {}
        -o, --work-icon <value>     Sets custom work icon/text. default: {}
        -b, --break-icon <value>    Sets custom break icon/text. default: {}

        --no-icons                  Disable the pause/play icon
        --no-work-icons             Disable the work/break icon

        --autow                     Starts a work cycle automatically after a break
        --autob                     Starts a break cycle automatically after work
        --persist                   Persist timer state between sessions

    operations:
        toggle                      Toggles the timer
        start                       Start the timer
        stop                        Stop the timer
        reset                       Reset timer to initial state

        set-work <value>            Set new work time
        set-short <value>           Set new short break time
        set-long <value>            Set new long break time"#,
        WORK_TIME / MINUTE,
        SHORT_BREAK_TIME / MINUTE,
        LONG_BREAK_TIME / MINUTE,
        PLAY_ICON,
        PAUSE_ICON,
        WORK_ICON,
        BREAK_ICON,
    );
}
