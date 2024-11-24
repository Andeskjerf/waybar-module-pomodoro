use serde::{Deserialize, Serialize};

use crate::{
    models::config::Config,
    utils::consts::{MAX_ITERATIONS, SLEEP_TIME},
};

use super::server::send_notification;

pub enum CycleType {
    Work,
    ShortBreak,
    LongBreak,
}

#[derive(Serialize, Deserialize)]
pub struct Timer {
    pub current_index: usize,
    pub elapsed_millis: u16,
    pub elapsed_time: u16,
    pub times: [u16; 3],
    pub iterations: u8,
    pub session_completed: u8,
    pub running: bool,
    socket_nr: i32,
}

impl Timer {
    pub fn new(work_time: u16, short_break: u16, long_break: u16, socker_nr: i32) -> Timer {
        Timer {
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

    pub fn reset(&mut self) {
        self.current_index = 0;
        self.elapsed_time = 0;
        self.elapsed_millis = 0;
        self.iterations = 0;
        self.running = false;
    }

    pub fn is_break(&self) -> bool {
        self.current_index != 0
    }

    pub fn set_time(&mut self, cycle: CycleType, input: u16) {
        self.reset();

        match cycle {
            CycleType::Work => self.times[0] = input * 60,
            CycleType::ShortBreak => self.times[1] = input * 60,
            CycleType::LongBreak => self.times[2] = input * 60,
        }
    }

    pub fn get_class(&self) -> String {
        // timer hasn't been started yet
        if self.elapsed_millis == 0
            && self.elapsed_time == 0
            && self.iterations == 0
            && self.session_completed == 0
        {
            "".to_owned()
        }
        // timer has been paused
        else if !self.running {
            "pause".to_owned()
        }
        // currently doing some work
        else if !self.is_break() {
            "work".to_owned()
        }
        // currently a break
        else if self.is_break() {
            "break".to_owned()
        } else {
            panic!("invalid condition occurred while setting class!");
        }
    }

    pub fn update_state(&mut self, config: &Config) {
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

    pub fn get_current_time(&self) -> u16 {
        self.times[self.current_index]
    }

    pub fn increment_time(&mut self) {
        self.elapsed_millis += SLEEP_TIME;
        if self.elapsed_millis >= 1000 {
            self.elapsed_millis = 0;
            self.elapsed_time += 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::utils::consts::{LONG_BREAK_TIME, SHORT_BREAK_TIME, SLEEP_DURATION, WORK_TIME};

    use super::*;

    fn create_timer() -> Timer {
        let socket_nr = 1;

        Timer::new(WORK_TIME, SHORT_BREAK_TIME, LONG_BREAK_TIME, socket_nr)
    }

    #[test]
    fn test_new_timer() {
        let timer = create_timer();

        assert_eq!(timer.current_index, 0);
        assert_eq!(timer.elapsed_millis, 0);
        assert_eq!(timer.elapsed_time, 0);
        assert_eq!(timer.times, [WORK_TIME, SHORT_BREAK_TIME, LONG_BREAK_TIME]);
        assert_eq!(timer.iterations, 0);
        assert_eq!(timer.session_completed, 0);
        assert!(!timer.running);
    }

    #[test]
    fn test_reset_timer() {
        let mut timer = create_timer();
        timer.current_index = 2;
        timer.elapsed_millis = 999;
        timer.elapsed_time = WORK_TIME - 1;
        timer.iterations = 4;
        timer.session_completed = 3;
        timer.running = true;

        timer.reset();

        assert_eq!(timer.current_index, 0);
        assert_eq!(timer.elapsed_millis, 0);
        assert_eq!(timer.elapsed_time, 0);
        assert_eq!(timer.iterations, 0);
        assert!(!timer.running);
    }

    #[test]
    fn test_is_break() {
        let mut timer = create_timer();

        assert!(!timer.is_break());

        timer.current_index = 1;
        assert!(timer.is_break());
    }

    #[test]
    fn test_set_time() {
        let mut timer = create_timer();

        timer.set_time(CycleType::Work, 30);
        assert_eq!(timer.times[0], 30 * 60);

        timer.set_time(CycleType::ShortBreak, 10);
        assert_eq!(timer.times[1], 10 * 60);

        timer.set_time(CycleType::LongBreak, 20);
        assert_eq!(timer.times[2], 20 * 60);
    }

    #[test]
    fn test_get_class() {
        let mut timer = create_timer();

        assert_eq!(timer.get_class(), "");

        timer.running = true;
        timer.elapsed_millis = 1;
        assert_eq!(timer.get_class(), "work");

        timer.current_index = 1;
        assert_eq!(timer.get_class(), "break");

        timer.running = false;
        assert_eq!(timer.get_class(), "pause");
    }

    #[test]
    fn test_update_state() {
        let mut timer = create_timer();
        let config = Config::default();

        // set to low times so the test passes faster
        let time = 1;
        timer.times[0] = time;
        timer.times[1] = time;
        timer.times[2] = time;

        // Initial state
        assert_eq!(timer.current_index, 0);
        assert_eq!(timer.iterations, 0);

        // Update state after work time is completed
        for _ in 0..time * 1000 / SLEEP_TIME {
            timer.increment_time();
            std::thread::sleep(SLEEP_DURATION);
        }
        timer.update_state(&config);
        assert_eq!(timer.current_index, 1); // Move to short break

        // Update state after short break is completed
        for _ in 0..time * 1000 / SLEEP_TIME {
            timer.increment_time();
            std::thread::sleep(SLEEP_DURATION);
        }
        timer.update_state(&config);

        // we need to trigger a long break
        timer.iterations = MAX_ITERATIONS - 1;

        // Update state after short break is completed
        for _ in 0..time * 1000 / SLEEP_TIME {
            timer.increment_time();
            std::thread::sleep(SLEEP_DURATION);
        }

        timer.update_state(&config);
        assert_eq!(timer.current_index, 2); // Move to long break
    }

    #[test]
    fn test_increment_elapsed_time() {
        let mut timer = create_timer();

        assert_eq!(timer.elapsed_millis, 0);
        assert_eq!(timer.elapsed_time, 0);

        timer.increment_time();
        assert_eq!(timer.elapsed_millis, SLEEP_TIME); // Assuming SLEEP_INTERVAL is defined
        assert_eq!(timer.elapsed_time, 0);

        for _ in 1..SLEEP_TIME {
            timer.increment_time();
        }
        assert_eq!(timer.elapsed_millis, 0);
        assert_eq!(timer.elapsed_time, 10);
    }
}
