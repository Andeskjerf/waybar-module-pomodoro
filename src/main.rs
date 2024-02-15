use std::time::Duration;

const SLEEP_TIME: u16 = 1;
const SLEEP_DURATION: Duration = Duration::from_secs(SLEEP_TIME as u64);

const MINUTE: u16 = 60;

fn format_time(elapsed_time: u16, max_time: u16) -> String {
    let time = max_time - elapsed_time;
    let minute = time / MINUTE;
    let second = time % MINUTE;
    format!("{:02}:{:02}", minute, second)
}

fn main() {
    let mut elapsed_time: u16 = 0;
    let max_time: u16 = 25 * MINUTE;
    loop {
        let value = format_time(elapsed_time, max_time);
        println!("{{\"text\": \"{}\"}}", value);

        elapsed_time += 1;
        std::thread::sleep(SLEEP_DURATION);
    }
}
