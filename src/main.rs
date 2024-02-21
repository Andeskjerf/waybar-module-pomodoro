use std::{
    env, fs,
    io::{Read, Write},
    os::unix::net::{UnixListener, UnixStream},
    path::Path,
    sync::mpsc::{Receiver, Sender},
    thread,
    time::Duration,
};

const SLEEP_TIME: u16 = 1;
const SLEEP_DURATION: Duration = Duration::from_secs(SLEEP_TIME as u64);
const MINUTE: u16 = 60;
const MAX_ITERATIONS: u8 = 4;
const WORK_TIME: u16 = 25 * MINUTE;
const SHORT_BREAK_TIME: u16 = 5 * MINUTE;
const LONG_BREAK_TIME: u16 = 15 * MINUTE;

struct State {
    current_index: usize,
    elapsed_time: u16,
    times: [u16; 3],
    iterations: u8,
    running: bool,
}

impl State {
    fn new() -> State {
        State {
            current_index: 0,
            elapsed_time: 0,
            times: [WORK_TIME, SHORT_BREAK_TIME, LONG_BREAK_TIME],
            iterations: 0,
            running: false,
        }
    }

    fn reset(&mut self) {
        self.current_index = 0;
        self.elapsed_time = 0;
        self.iterations = 0;
        self.running = false;
    }

    fn update_state(&mut self) {
        if (self.times[self.current_index] - self.elapsed_time) == 0 {
            // we don't want to get the last break time until we've done 4 pomodoro cycles
            self.current_index = (self.current_index + 1) % 2;
            self.elapsed_time = 0;
            // stop the timer and wait for user to start next cycle
            self.running = false;
            // only increment iterations once we've had a short break and are back to work
            if self.current_index == 0 || self.iterations == MAX_ITERATIONS - 1 {
                self.iterations += 1;
            }
        }

        // if we've done 4 pomodoro cycles, reset iterations and do a long break
        if self.iterations == MAX_ITERATIONS {
            self.iterations = 0;
            self.current_index = self.times.len() - 1
        }
    }

    fn get_current_time(&self) -> u16 {
        self.times[self.current_index]
    }

    fn increment_time(&mut self) {
        self.elapsed_time += 1;
    }
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

fn handle_client(rx: Receiver<String>) {
    let mut state = State::new();

    loop {
        match rx.try_recv() {
            Ok(message) => match message.as_str() {
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
            },
            Err(_) => {}
        }

        let value = format_time(state.elapsed_time, state.get_current_time());
        let value_prefix = if state.running { "⏸ " } else { "▶ " };
        let tooltip = if state.running { "Running" } else { "Stopped" };
        let class = if state.current_index == 0 {
            "work"
        } else {
            "break"
        };
        state.update_state();
        print_message(
            value_prefix.clone().to_string() + value.clone().as_str(),
            tooltip,
            class,
        );

        if state.running {
            state.increment_time();
        }

        std::thread::sleep(SLEEP_DURATION);
    }
}

fn spawn_server(socket_path: &String) {
    // remove old socket if it exists
    if Path::new(&socket_path).exists() {
        fs::remove_file(&socket_path).unwrap();
    }

    let listener = UnixListener::bind(&socket_path).unwrap();
    let (tx, rx): (Sender<String>, Receiver<String>) = std::sync::mpsc::channel();
    thread::spawn(|| handle_client(rx));

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                // read incoming data
                let mut message = String::new();
                stream
                    .read_to_string(&mut message)
                    .expect("Failed to read UNIX stream");
                tx.send(message.clone()).unwrap();
            }
            Err(err) => println!("Error: {}", err),
        }
    }
}

fn main() -> std::io::Result<()> {
    let socket_path: String = format!(
        "{}/{}.socket",
        env::temp_dir().display().to_string(),
        "waybar-module-pomodoro"
    );
    let args: Vec<String> = env::args().collect();

    if args.len() == 1 {
        spawn_server(&socket_path);
        return Ok(());
    }

    let mut stream = UnixStream::connect(&socket_path)?;
    let opt = &args[1];
    stream.write_all(opt.as_bytes())?;

    Ok(())
}
