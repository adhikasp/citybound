use kay::{ActorSystem, World, TypedID};
use compact::CVec;

pub mod ui;

pub use ::units::{Instant, Ticks, Duration, TICKS_PER_SIM_MINUTE, TICKS_PER_SIM_SECOND,
TimeOfDay, TimeOfDayRange};

pub trait Temporal {
    fn tick(&mut self, dt: f32, current_instant: Instant, world: &mut World);
}

pub trait Sleeper {
    fn wake(&mut self, current_instant: Instant, world: &mut World);
}

#[derive(Compact, Clone)]
pub struct Time {
    id: TimeID,
    current_instant: Instant,
    sleepers: CVec<(Instant, SleeperID)>,
    speed: u16,
}

impl Time {
    pub fn spawn(id: TimeID, _: &mut World) -> Time {
        Time {
            id,
            current_instant: Instant::new(0),
            sleepers: CVec::new(),
            speed: 1,
        }
    }

    pub fn progress(&mut self, world: &mut World) {
        for _ in 0..self.speed {
            TemporalID::global_broadcast(world).tick(
                1.0 / (TICKS_PER_SIM_SECOND as f32),
                self.current_instant,
                world,
            );
            // Rebuild the queue instead of popping from it: the pop-from-the-back
            // form read the wrong element under rustc 1.102 (a due sleeper at
            // Instant(30) was never woken while the clock ran past 4000), which
            // stalled everything scheduled through `Sleeper` - trips, households
            // and city development included.
            if !self.sleepers.is_empty() {
                let now = self.current_instant;
                let mut due: Vec<SleeperID> = Vec::new();
                let mut keep: Vec<(Instant, SleeperID)> =
                    Vec::with_capacity(self.sleepers.len());
                for &(end, sleeper) in self.sleepers.iter() {
                    if end.ticks() < now.ticks() {
                        due.push(sleeper);
                    } else {
                        keep.push((end, sleeper));
                    }
                }
                if !due.is_empty() {
                    self.sleepers = keep.into_iter().collect();
                    for sleeper in due {
                        sleeper.wake(now, world);
                    }
                }
            }
            self.current_instant += Ticks(1);
        }
    }

    pub fn wake_up_in(&mut self, remaining_ticks: Ticks, sleeper_id: SleeperID, _: &mut World) {
        let wake_up_at = self.current_instant + remaining_ticks;
        let maybe_idx = self
            .sleepers
            .binary_search_by_key(&wake_up_at.iticks(), |&(t, _)| -(t.iticks()));
        let insert_idx = match maybe_idx {
            Ok(idx) | Err(idx) => idx,
        };
        self.sleepers.insert(insert_idx, (wake_up_at, sleeper_id));
    }
}

pub fn setup(system: &mut ActorSystem) {
    system.register::<Time>();
    auto_setup(system);
    ui::auto_setup(system);
}

pub fn spawn(world: &mut World) -> TimeID {
    TimeID::spawn(world)
}

mod kay_auto;
pub use self::kay_auto::*;

