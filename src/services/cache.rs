use std::{env, error::Error, fs::File, io::Write, path::PathBuf};

use crate::models::config::Config;

use super::timer::Timer;

const MODULE: &str = env!("CARGO_PKG_NAME");
const VERSION: &str = env!("CARGO_PKG_VERSION");

pub fn store(state: &Timer) -> Result<(), Box<dyn Error>> {
    let mut filepath = cache_dir()?;
    let output_name = format!("{}-{}", MODULE, VERSION);
    filepath.push(output_name);

    let data = serde_json::to_string(&state).expect("Not a serializable type");
    Ok(File::create(filepath)?.write_all(data.as_bytes())?)
}

pub fn restore(state: &mut Timer, config: &Config) -> Result<(), Box<dyn Error>> {
    let mut filepath = cache_dir()?;
    let output_name = format!("{}-{}", MODULE, VERSION);
    filepath.push(output_name);

    let file = File::open(filepath)?;
    let json: serde_json::Value = serde_json::from_reader(file)?;
    let restored: Timer = serde_json::from_value(json)?;

    if match_timers(config, &restored.times) {
        state.current_index = restored.current_index;
        state.elapsed_millis = restored.elapsed_millis;
        state.elapsed_time = restored.elapsed_time;
        state.times = restored.times;
        state.iterations = restored.iterations;
        state.session_completed = restored.session_completed;
    }

    Ok(())
}

fn match_timers(config: &Config, times: &[u16; 3]) -> bool {
    let work_time: u16 = times[0];
    let short_break: u16 = times[1];
    let long_break: u16 = times[2];

    if config.work_time != work_time
        || config.short_break != short_break
        || config.long_break != long_break
    {
        return false;
    }

    true
}

fn cache_dir() -> Result<PathBuf, Box<dyn Error>> {
    let mut dir = if let Some(dir) = dirs::cache_dir() {
        dir
    } else {
        return Err("unable to get cache dir".into());
    };

    dir.push(MODULE);
    if let Err(e) = std::fs::create_dir_all(&dir) {
        println!("create_dir: path == {:?}, err == {e}", dir);
    }
    Ok(dir)
}

#[cfg(test)]
mod tests {
    use super::*;

    // Mock the env variables used in the code
    const MODULE: &str = "waybar-module-pomodoro";
    const VERSION: &str = "0.1";

    fn create_timer(
        work_time: Option<u16>,
        short_break: Option<u16>,
        long_break: Option<u16>,
    ) -> Timer {
        Timer {
            current_index: 1,
            elapsed_millis: 950,
            elapsed_time: 300,
            times: [
                work_time.unwrap_or(25),
                short_break.unwrap_or(5),
                long_break.unwrap_or(15),
            ],
            iterations: 2,
            session_completed: 8,
            running: false,
            socket_nr: 0,
        }
    }

    #[test]
    fn test_store_and_restore() -> Result<(), Box<dyn Error>> {
        unsafe {
            std::env::set_var("CARGO_PKG_NAME", MODULE);
            std::env::set_var("CARGO_PKG_VERSION", VERSION);
        }

        let timer = create_timer(None, None, None);
        store(&timer)?;
        let mut restored_timer = create_timer(Some(30), Some(10), Some(20));

        let config = Config {
            work_time: 25,
            short_break: 5,
            long_break: 15,
            ..Default::default()
        };

        restore(&mut restored_timer, &config)?;

        assert_eq!(restored_timer.current_index, timer.current_index);
        assert_eq!(restored_timer.elapsed_millis, timer.elapsed_millis);
        assert_eq!(restored_timer.elapsed_time, timer.elapsed_time);
        assert_eq!(restored_timer.times, timer.times);
        assert_eq!(restored_timer.iterations, timer.iterations);
        assert_eq!(restored_timer.session_completed, timer.session_completed);

        Ok(())
    }

    #[test]
    fn test_store_and_restore_mismatched_config() -> Result<(), Box<dyn Error>> {
        unsafe {
            std::env::set_var("CARGO_PKG_NAME", MODULE);
            std::env::set_var("CARGO_PKG_VERSION", VERSION);
        }

        let timer = create_timer(None, None, None);
        store(&timer)?;
        let mut restored_timer = create_timer(Some(30), Some(10), Some(20));

        let config = Config {
            work_time: 30,
            short_break: 10,
            long_break: 20,
            ..Default::default()
        };

        restore(&mut restored_timer, &config)?;

        // Check if the restored timer state is not changed
        assert_eq!(restored_timer.current_index, 1);
        assert_eq!(restored_timer.elapsed_millis, 950);
        assert_eq!(restored_timer.elapsed_time, 300);
        assert_eq!(restored_timer.times, [30, 10, 20]);
        assert_eq!(restored_timer.iterations, 2);
        assert_eq!(restored_timer.session_completed, 8);

        Ok(())
    }

    #[test]
    fn test_cache_dir_creation() -> Result<(), Box<dyn Error>> {
        unsafe {
            std::env::set_var("CARGO_PKG_NAME", MODULE);
            std::env::set_var("CARGO_PKG_VERSION", VERSION);
        }

        let mut dir = dirs::cache_dir().expect("unable to get cache dir");
        dir.push(MODULE);
        if let Err(e) = std::fs::create_dir(&dir) {
            println!("err: err == {e}");
        }

        let result = cache_dir()?;

        assert_eq!(result, dir);
        assert!(dir.is_dir());

        Ok(())
    }

    #[test]
    fn test_match_timers_match() {
        let config = Config {
            work_time: 25,
            short_break: 5,
            long_break: 15,
            ..Default::default()
        };

        let times = [25, 5, 15];

        assert!(match_timers(&config, &times));
    }

    #[test]
    fn test_match_timers_mismatch() {
        let config = Config {
            work_time: 30,
            short_break: 10,
            long_break: 20,
            ..Default::default()
        };

        let times = [25, 5, 15];

        assert!(!match_timers(&config, &times));
    }
}
