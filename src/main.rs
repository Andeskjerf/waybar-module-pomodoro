use std::time::Duration;

const SLEEP_TIME: u16 = 1;
const SLEEP_DURATION: Duration = Duration::from_secs(SLEEP_TIME as u64);
const MINUTE: u16 = 60;
const MAX_ITERATIONS: u8 = 4;

struct State {
    current_index: usize,
    elapsed_time: u16,
    times: [u16; 3],
    iterations: u8,
}

impl State {
    fn new() -> State {
        State {
            current_index: 0,
            elapsed_time: 0,
            // work time, break time, rest time
            times: [1 * MINUTE, 1 * MINUTE, 15 * MINUTE],
            iterations: 0,
        }
    }

    fn update_state(&mut self) {
        if (self.times[self.current_index] - self.elapsed_time) == 0 {
            // we don't want to get the last break time until we've done 4 pomodoro cycles
            self.current_index = (self.current_index + 1) % 2;
            self.elapsed_time = 0;
            // only increment iterations once we've had a short break and are back to work
            if self.current_index == 0 {
                self.iterations += 1;
            }
            println!("Iterations: {}", self.iterations);
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

fn main() {
    let mut state = State::new();

    loop {
        state.update_state();

        let value = format_time(state.elapsed_time, state.get_current_time());
        println!("{{\"text\": \"{}\"}}", value);

        state.increment_time();
        std::thread::sleep(SLEEP_DURATION);
    }
}
