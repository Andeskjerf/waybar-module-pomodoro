use models::config::{parse_set_operations, Config, OPERATIONS};
use services::server::{get_existing_sockets, send_message_socket, spawn_server};
use signal_hook::{
    consts::{SIGHUP, SIGINT, SIGTERM},
    iterator::Signals,
};
use std::{env, thread};
use utils::consts::{
    BREAK_ICON, LONG_BREAK_TIME, MAX_ITERATIONS, MINUTE, PAUSE_ICON, PLAY_ICON, SHORT_BREAK_TIME,
    WORK_ICON, WORK_TIME,
};

mod models;
mod services;
mod utils;

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

fn print_help() {
    println!(
        r#"usage: waybar-module-pomodoro [options] [operation]
    options:
        -h, --help                  Prints this help message
        -v, --version               Prints the version string
        -w, --work <value>          Sets how long a work cycle is, in minutes. default: {}
        -s, --shortbreak <value>    Sets how long a short break is, in minutes. default: {}
        -l, --longbreak <value>     Sets how long a long break is, in minutes. default: {}
        -i, --intervals <value>     How many intervals there should be before a long break. default: {}

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
        MAX_ITERATIONS,
        PLAY_ICON,
        PAUSE_ICON,
        WORK_ICON,
        BREAK_ICON,
    );
}
