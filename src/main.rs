use config::Config;
use notify_rust::Notification;
use signal_hook::{
    consts::{SIGINT, SIGTERM},
    iterator::Signals,
};
use std::{
    env, fs,
    io::{Error, Read, Write},
    os::unix::net::{UnixListener, UnixStream},
    path::Path,
    sync::mpsc::{Receiver, Sender},
    thread,
    time::Duration,
};

mod config;

const SLEEP_TIME: u16 = 100;
const SLEEP_DURATION: Duration = Duration::from_millis(SLEEP_TIME as u64);
const MINUTE: u16 = 60;
const MAX_ITERATIONS: u8 = 4;
const WORK_TIME: u16 = 25 * MINUTE;
const SHORT_BREAK_TIME: u16 = 5 * MINUTE;
const LONG_BREAK_TIME: u16 = 15 * MINUTE;
// const PLAY_ICON: &str = "";
// const PAUSE_ICON: &str = "";
const PLAY_ICON: &str = "▶";
const PAUSE_ICON: &str = "⏸";
const WORK_ICON: &str = "󰔟";
const BREAK_ICON: &str = "";

enum CycleType {
    Work,
    ShortBreak,
    LongBreak,
}

struct State {
    current_index: usize,
    elapsed_millis: u16,
    elapsed_time: u16,
    times: [u16; 3],
    iterations: u8,
    session_completed: u8,
    running: bool,
    socket_nr: i32,
}

impl State {
    fn new(work_time: u16, short_break: u16, long_break: u16, socker_nr: i32) -> State {
        State {
            current_index: 0,
            elapsed_millis: 0,
            elapsed_time: 0,
            times: [work_time, short_break, long_break],
            iterations: 0,
            session_completed: 0,
            running: false,
            socket_nr: socker_nr,
        }
    }

    fn reset(&mut self) {
        self.current_index = 0;
        self.elapsed_time = 0;
        self.elapsed_millis = 0;
        self.iterations = 0;
        self.running = false;
    }

    fn is_break(&self) -> bool {
        self.current_index != 0
    }

    fn update_state(&mut self, config: &Config) {
        if (self.times[self.current_index] - self.elapsed_time) == 0 {
            // if we're on the third iteration and first work, then we want a long break
            if self.current_index == 0 && self.iterations == MAX_ITERATIONS - 1 {
                self.current_index = self.times.len() - 1;
                self.iterations = MAX_ITERATIONS;
            }
            // if we've had our long break, reset everything and start over
            else if self.current_index == self.times.len() - 1
                && self.iterations == MAX_ITERATIONS
            {
                self.current_index = 0;
                self.iterations = 0;
                // since we've gone through a long break, we've also completed a single pomodoro!
                self.session_completed += 1;
            }
            // otherwise, run as normal
            else {
                self.current_index = (self.current_index + 1) % 2;
                if self.current_index == 0 {
                    self.iterations += 1;
                }
            }

            self.elapsed_time = 0;

            // if the user has passed either auto flag, we want to keep ticking the timer
            // NOTE: the is_break() seems to be flipped..?
            self.running = (config.autob && self.is_break()) || (config.autow && !self.is_break());

            // only send a notification for the first instance of the module
            if self.socket_nr == 0 {
                send_notification(match self.current_index {
                    0 => CycleType::Work,
                    1 => CycleType::ShortBreak,
                    2 => CycleType::LongBreak,
                    _ => panic!("Invalid cycle type"),
                });
            }
        }
    }

    fn get_current_time(&self) -> u16 {
        self.times[self.current_index]
    }

    fn increment_time(&mut self) {
        self.elapsed_millis += SLEEP_TIME;
        if self.elapsed_millis >= 1000 {
            self.elapsed_millis = 0;
            self.elapsed_time += 1;
        }
    }
}

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

fn get_class(state: &State) -> String {
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

fn handle_client(rx: Receiver<String>, socket_path: String, config: Config) {
    let socket_nr = socket_path
        .chars()
        .filter_map(|c| c.to_digit(10))
        .fold(0, |acc, digit| acc * 10 + digit) as i32;

    let mut state = State::new(
        config.work_time,
        config.short_break,
        config.long_break,
        socket_nr,
    );

    loop {
        if let Ok(message) = rx.try_recv() {
            match message.as_str() {
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

        let value = format_time(state.elapsed_time, state.get_current_time());
        let value_prefix = if !config.no_icons {
            if state.running {
                &config.pause_icon
            } else {
                &config.play_icon
            }
        } else {
            ""
        };
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
        let cycle_icon = if !state.is_break() {
            &config.work_icon
        } else {
            &config.break_icon
        };
        state.update_state(&config);
        print_message(
            format!("{} {} {}", value_prefix, value, cycle_icon),
            tooltip.as_str(),
            class,
        );

        if state.running {
            state.increment_time();
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
                tx.send(message.clone()).unwrap();
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

    let mut signals = Signals::new([SIGINT, SIGTERM]).unwrap();
    thread::spawn(move || {
        for _ in signals.forever() {
            send_message_socket(&socket_path, "exit").expect("unable to send message to server");
        }
    });
}

fn main() -> std::io::Result<()> {
    // valid operations
    let operations = ["toggle", "start", "stop", "reset"];

    let options = env::args().collect::<Vec<String>>();
    if options.contains(&"--help".to_string()) || options.contains(&"-h".to_string()) {
        print_help();
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
        .filter(|x| operations.contains(&x.as_str()))
        .collect::<Vec<String>>();

    if operation.is_empty() {
        sockets.push(socket_path.clone());
        process_signals(socket_path.clone());
        spawn_server(&socket_path, config);
        return Ok(());
    }

    for socket in sockets {
        send_message_socket(&socket, &operation[0])?;
    }
    Ok(())
}

fn print_help() {
    println!(
        r#"usage: waybar-module-pomodoro [options] [operation]
    options:
        -h, --help                  Prints this help message
        -w, --work <value>          Sets how long a work cycle is, in minutes. default: {}
        -s, --shortbreak <value>    Sets how long a short break is, in minutes. default: {}
        -l, --longbreak <value>     Sets how long a long break is, in minutes. default: {}

        -p, --play <value>          Sets custom play icon/text. default: {}
        -a, --pause <value>         Sets custom pause icon/text. default: {}
        -o, --work-icon <value>     Sets custom work icon/text. default: {}
        -b, --break-icon <value>    Sets custom break icon/text. default: {}

        --no-icons                  Disable the pause/play icon

        --autow                     Starts a work cycle automatically after a break
        --autob                     Starts a break cycle automatically after work

    operations:
        toggle                      Toggles the timer
        start                       Start the timer
        pause                       Pause the timer
        reset                       Reset timer to initial state"#,
        WORK_TIME / MINUTE,
        SHORT_BREAK_TIME / MINUTE,
        LONG_BREAK_TIME / MINUTE,
        PLAY_ICON,
        PAUSE_ICON,
        WORK_ICON,
        BREAK_ICON,
    );
}
