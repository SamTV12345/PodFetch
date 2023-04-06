use std::thread::sleep;
use std::time::Duration;

pub fn do_retry<F, T, E>(mut func: F) -> Result<T, E>
    where
        F: FnMut() -> Result<T, E>,
{
    let mut tries = 0;

    loop {
        match func() {
            ok @ Ok(_) => return ok,
            err @ Err(_) => {
                tries += 1;

                if tries >= 10 {
                    return err;
                }
               sleep(Duration::from_millis(500))
            }
        }
    }
}