use kay::{World, ActorSystem, Actor, TypedID};
use wasm_bindgen::prelude::*;

use js;
use SYSTEM;
use browser_utils::{FrameListener, FrameListenerID};

use cb_time::actors::TimeID;
use cb_time::actors::ui::{TimeUI, TimeUIID};
use cb_time::units::{TimeOfDay, Instant};

#[wasm_bindgen]
pub fn set_sim_speed(new_speed: u16) {
    let system = unsafe { &mut *SYSTEM };
    let world = &mut system.world();
    TimeID::global_first(world).set_speed(new_speed, world);
}

#[derive(Compact, Clone)]
pub struct BrowserTimeUI {
    id: BrowserTimeUIID,
}

impl BrowserTimeUI {
    pub fn spawn(id: BrowserTimeUIID, _: &mut World) -> BrowserTimeUI {
        BrowserTimeUI { id }
    }
}

impl FrameListener for BrowserTimeUI {
    fn on_frame(&mut self, world: &mut World) {
        TimeID::global_first(world).get_info(self.id_as(), world);
    }
}

impl TimeUI for BrowserTimeUI {
    fn on_time_info(&mut self, current_instant: Instant, speed: u16, _world: &mut World) {
        js::call(
            "timeInfo",
            &[
                JsValue::from_f64(current_instant.ticks() as f64),
                js::to_js(&TimeOfDay::from(current_instant).hours_minutes()),
                JsValue::from_f64(speed as f64),
            ],
        );
    }
}

mod kay_auto;
pub use self::kay_auto::*;

pub fn setup(system: &mut ActorSystem) {
    system.register::<BrowserTimeUI>();
    auto_setup(system);
}

pub fn spawn(world: &mut World) {
    BrowserTimeUIID::spawn(world);
}
