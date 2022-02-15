use crate::engine::clock::Clock;

use std::time::Duration;
use thiserror::Error;

static mut SLEEP_TOLERANCE: Duration = Duration::from_micros(0);

pub(crate) fn get_sleep_tolerance() -> Duration {
    unsafe { SLEEP_TOLERANCE }
}

#[derive(Debug, Error)]
pub enum SleepError {
    #[error("sleep exceeded target duration by {:?}", .0)]
    TargetDurationExceeded(Duration),
}

pub fn sleep(duration: Duration) -> Result<(), SleepError> {
    let mut clock = Clock::default();
    clock.tick();

    let tolerance = unsafe { SLEEP_TOLERANCE };

    if tolerance < duration {
        if duration - tolerance > Duration::from_secs_f32(0.0) {
            std::thread::sleep(duration - tolerance);
        }

        let elapsed = clock.elapsed();
        if elapsed > duration {
            unsafe {
                SLEEP_TOLERANCE += Duration::from_micros(100);
            }
            return Err(SleepError::TargetDurationExceeded(elapsed - duration));
        }
    }

    while clock.elapsed() < duration {
        // Eat CPU cycles.
    }

    Ok(())
}
