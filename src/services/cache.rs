use std::{
    env,
    fs::File,
    io::{self, Write},
    path::{Path, PathBuf},
};

use crate::models::config::Config;

use super::timer::Timer;

const MODULE: &str = env!("CARGO_PKG_NAME");
const VERSION: &str = env!("CARGO_PKG_VERSION");

pub(crate) fn store(state: &Timer) -> io::Result<()> {
    let mut filepath = cache_dir()?;
    let output_name = format!("{}-{}", MODULE, VERSION);
    filepath.push(output_name);

    let data = serde_json::to_string(&state).expect("Not a serializable type");
    File::create(filepath)?.write_all(data.as_bytes())
}

pub(crate) fn restore(state: &mut Timer, config: &Config) -> io::Result<()> {
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

fn create_dir(p: &Path) -> io::Result<()> {
    println!("{:?}", p);
    if !p.is_dir() {
        std::fs::create_dir(p)
    } else {
        Ok(())
    }
}

fn cache_dir() -> io::Result<PathBuf> {
    if let Ok(path) = env::var("XDG_CACHE_HOME") {
        let mut path: PathBuf = path.into();
        path.push(MODULE);
        create_dir(&path)?;
        Ok(path)
    } else if let Ok(path) = env::var("HOME") {
        let mut path: PathBuf = path.into();
        path.push(".cache");
        path.push(MODULE);
        create_dir(&path)?;
        Ok(path)
    } else {
        Err(io::Error::new(
            io::ErrorKind::Other,
            "failed to read both $XDG_CACHE_HOME and $HOME environment variables".to_string(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use crate::utils::consts::{LONG_BREAK_TIME, SHORT_BREAK_TIME, WORK_TIME};

    use super::*;
    use std::{fs, str::FromStr};

    fn create_timer() -> Timer {
        let mut timer = Timer::new(WORK_TIME, SHORT_BREAK_TIME, LONG_BREAK_TIME, 0);
        timer.elapsed_millis = 12345;
        timer.elapsed_time = 50;
        timer.iterations = 2;
        timer.session_completed = 8;
        timer
    }

    #[test]
    fn test_store() -> io::Result<()> {
        // Arrange
        let mut temp_dir = env::temp_dir();
        temp_dir.push("pomodoro_test");
        fs::create_dir_all(&temp_dir)?;

        let timer = create_timer();

        unsafe {
            env::set_var("XDG_CACHE_HOME", temp_dir.to_str().unwrap());
        }

        // Act
        store(&timer)?;

        // Assert
        let mut filepath = cache_dir()?;
        filepath.push(format!("{}-{}", MODULE, VERSION));
        assert!(filepath.exists());
        let file_content = fs::read_to_string(filepath)?;
        let restored_timer: Timer =
            serde_json::from_str(&file_content).expect("Failed to deserialize");
        assert_eq!(timer, restored_timer);

        // Cleanup
        fs::remove_dir_all(temp_dir)?;

        Ok(())
    }

    #[test]
    fn test_restore() -> io::Result<()> {
        // Arrange
        let mut temp_dir = env::temp_dir();
        temp_dir.push("pomodoro_test");
        fs::create_dir_all(&temp_dir)?;

        let timer = create_timer();

        let mut filepath = temp_dir.clone();
        filepath.push(format!("{}-{}", MODULE, VERSION));
        let serialized_timer = serde_json::to_string(&timer).expect("Failed to serialize");
        let mut file = File::create(filepath)?;
        file.write_all(serialized_timer.as_bytes())?;

        unsafe {
            env::set_var("XDG_CACHE_HOME", temp_dir.to_str().unwrap());
        }

        let config = Config::default();

        let mut state = Timer::default();

        // Act
        restore(&mut state, &config)?;

        // Assert
        assert_eq!(timer, state);

        // Cleanup
        fs::remove_dir_all(temp_dir)?;

        Ok(())
    }

    #[test]
    fn test_restore_mismatched_config() -> io::Result<()> {
        // Arrange
        let mut temp_dir = env::temp_dir();
        temp_dir.push("pomodoro_test");
        fs::create_dir_all(&temp_dir)?;

        let timer = create_timer();

        let mut filepath = temp_dir.clone();
        filepath.push(format!("{}-{}", MODULE, VERSION));
        let serialized_timer = serde_json::to_string(&timer).expect("Failed to serialize");
        let mut file = File::create(filepath)?;
        file.write_all(serialized_timer.as_bytes())?;

        unsafe {
            env::set_var("XDG_CACHE_HOME", temp_dir.to_str().unwrap());
        }

        // Mismatched config
        let config = Config {
            work_time: 30,
            short_break: 10,
            long_break: 20,
            ..Default::default()
        };

        let mut state = Timer::default();

        // Act & Assert
        assert!(restore(&mut state, &config).is_ok());

        // State should not be updated if times do not match
        assert_eq!(Timer::default(), state);

        // Cleanup
        fs::remove_dir_all(temp_dir)?;

        Ok(())
    }

    #[test]
    fn test_cache_dir_xdg() -> io::Result<()> {
        // Arrange
        let temp_dir = env::temp_dir();
        unsafe {
            env::set_var("XDG_CACHE_HOME", temp_dir.to_str().unwrap());
        }

        // Act
        let cache_path = cache_dir()?;

        // Assert
        assert_eq!(cache_path, temp_dir.join(MODULE));

        Ok(())
    }

    #[test]
    fn test_cache_dir_home() -> io::Result<()> {
        // Arrange
        let temp_dir = dirs::home_dir().unwrap();
        unsafe {
            env::set_var("HOME", temp_dir.to_str().unwrap());
            env::remove_var("XDG_CACHE_HOME");
        }

        // Act
        let cache_path = cache_dir()?;

        // Assert
        assert_eq!(cache_path, temp_dir.join(".cache").join(MODULE));

        Ok(())
    }

    #[test]
    fn test_cache_dir_failure() -> io::Result<()> {
        // Arrange
        unsafe {
            env::remove_var("XDG_CACHE_HOME");
            env::remove_var("HOME");
        }

        // Act & Assert
        assert!(cache_dir().is_err());

        Ok(())
    }
}
