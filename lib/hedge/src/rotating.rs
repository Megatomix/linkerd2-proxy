extern crate tokio_timer;

use std::mem;
use std::time::{Duration, Instant};

use tokio_timer::clock;

/// This represents a "rotating" value which stores two T values, one which
/// should be read and one which should be written to.  Every period, the
/// read T is discarded and replaced by the write T.  The idea here is that
/// the read T should always contain a full period (the previous period) of
/// write operations.
pub struct Rotating<T> {
    read: T,
    write: T,
    last_rotation: Instant,
    period: Duration,
}

pub trait Clear {
    fn clear(&mut self);
}

impl<T> Rotating<T>
where
    T: Clear,
{
    pub fn new(period: Duration, new: fn() -> T) -> Rotating<T> {
        Rotating {
            read: new(),
            write: new(),
            last_rotation: clock::now(),
            period,
        }
    }

    pub fn read(&mut self) -> &mut T {
        self.maybe_rotate();
        &mut self.read
    }

    pub fn write(&mut self) -> &mut T {
        self.maybe_rotate();
        &mut self.write
    }

    fn maybe_rotate(&mut self) {
        let delta = clock::now() - self.last_rotation;
        // TODO: replace with delta.duration_div when it becomes stable.
        let rotations =
            (Self::duration_as_nanos(&delta) / Self::duration_as_nanos(&self.period)) as u32;
        if rotations >= 2 {
            self.clear();
        } else if rotations == 1 {
            self.rotate();
        }
        self.last_rotation += self.period * rotations;
    }

    fn rotate(&mut self) {
        mem::swap(&mut self.read, &mut self.write);
        self.write.clear();
    }

    fn clear(&mut self) {
        self.read.clear();
        self.write.clear();
    }

    fn duration_as_nanos(d: &Duration) -> u64 {
        d.as_secs() * 1_000_000_000 + (d.subsec_nanos() as u64)
    }
}
