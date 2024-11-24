use std::time::Duration;

pub const SLEEP_TIME: u16 = 100;
pub const SLEEP_DURATION: Duration = Duration::from_millis(SLEEP_TIME as u64);
pub const MINUTE: u16 = 60;
pub const MAX_ITERATIONS: u8 = 4;
pub const WORK_TIME: u16 = 25 * MINUTE;
pub const SHORT_BREAK_TIME: u16 = 5 * MINUTE;
pub const LONG_BREAK_TIME: u16 = 15 * MINUTE;
pub const PLAY_ICON: &str = "▶";
pub const PAUSE_ICON: &str = "⏸";
pub const WORK_ICON: &str = "󰔟";
pub const BREAK_ICON: &str = "";
