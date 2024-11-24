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
    if !dir.is_dir() {
        std::fs::create_dir(&dir)?;
    }
    Ok(dir)
}

#[cfg(test)]
mod tests {
    use crate::utils::consts::{LONG_BREAK_TIME, SHORT_BREAK_TIME, WORK_TIME};

    use super::*;
    use std::fs;
}
