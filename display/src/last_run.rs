use std::fs::File;
use std::io::Error;
use std::path::Path;
use std::time::{Duration, SystemTime};

const LOCK_FILE: &str = "/var/lock/eink-display.lock";

pub fn safe_to_run() -> bool {
    /// Returns true if the lock file was modified greater than 2 minutes ago.
    /// In any failure, assumes that it's safe. The consequences of running too soon are minimal, and running at the
    /// same time should be prevented by other means.
    if let Ok(metadata) = Path::new(LOCK_FILE).metadata() {
        if let Ok(modified) = metadata.modified() {
            if let Ok(since) = SystemTime::now().duration_since(modified) {
                return since > Duration::from_secs(2 * 60);
            }
        }
    }
    true
}

pub fn update_last_run() -> Result<(), Error> {
    /// Create the lock file if it doesn't exist, and explicitly set the modified timestamp to now. (probably unnecessary)
    let f = File::create(LOCK_FILE)?;
    f.set_modified(SystemTime::now())?;
    Ok(())
}
