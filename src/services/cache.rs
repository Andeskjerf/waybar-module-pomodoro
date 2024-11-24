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
