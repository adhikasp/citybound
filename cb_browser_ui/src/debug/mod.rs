use kay::TypedID;
use wasm_bindgen::prelude::*;

use js;
use SYSTEM;

use cb_planning::GestureID;
use cb_planning::plan_manager::ProjectID;
use planning::{CBPlanManagerID, CBGestureIntent};
use transport::transport_planning::RoadLaneConfig;
use descartes::{Corner};

#[wasm_bindgen]
pub fn plan_grid(project_id: JsValue, n: f64, n_lanes: u8, spacing: f32) {
    let project_id: ProjectID = match js::from_js(project_id) {
        Some(project_id) => project_id,
        None => return,
    };
    let n = n as isize;

    let system = unsafe { &mut *SYSTEM };
    let world = &mut system.world();

    let plan_manager = CBPlanManagerID::global_first(world);

    use ::transport::transport_planning::RoadIntent;
    use ::descartes::P2;

    for x in -n / 2..n / 2 {
        let id = GestureID::new();
        let p1 = P2::new(x as f32 * spacing, (-n / 2) as f32 * spacing);
        let p2 = P2::new(x as f32 * spacing, (n / 2) as f32 * spacing);
        let d = (p2 - p1).normalize();
        plan_manager.start_new_gesture(
            project_id,
            id,
            CBGestureIntent::Road(RoadIntent::new(
                vec![Corner::new(p1, Some(d), Some(d)), Corner::new(p2, Some(d), Some(d))],
                RoadLaneConfig {
                    n_lanes_forward: n_lanes,
                    n_lanes_backward: n_lanes,
                },
            )),
            world,
        );
    }

    for y in -n / 2..n / 2 {
        let id = GestureID::new();
        let p1 = P2::new((-n / 2) as f32 * spacing, y as f32 * spacing);
        let p2 = P2::new((n / 2) as f32 * spacing, y as f32 * spacing);
        let d = (p2 - p1).normalize();
        plan_manager.start_new_gesture(
            project_id,
            id,
            CBGestureIntent::Road(RoadIntent::new(
                vec![Corner::new(p1, Some(d), Some(d)), Corner::new(p2, Some(d), Some(d))],
                RoadLaneConfig {
                    n_lanes_forward: n_lanes,
                    n_lanes_backward: n_lanes,
                },
            )),
            world,
        );
    }
}

#[wasm_bindgen]
pub fn spawn_cars(tries_per_lane: usize) {
    let system = unsafe { &mut *SYSTEM };
    let world = &mut system.world();
    for _ in 0..tries_per_lane {
        ::transport::lane::LaneID::global_broadcast(world).manually_spawn_car_add_lane(world);
    }
}

use kay::{World, ActorSystem};
use compact::{CVec, CString};
use cb_util::log::{LogID, LogRecipient, LogRecipientID, Entry};

#[derive(Compact, Clone)]
pub struct LogUI {
    id: LogUIID,
}

impl LogUI {
    pub fn spawn(id: LogUIID, _: &mut World) -> LogUI {
        LogUI { id }
    }
}

impl LogRecipient for LogUI {
    fn receive_newest_logs(
        &mut self,
        entries: &CVec<Entry>,
        text: &CString,
        effective_last: u32,
        effective_text_start: u32,
        _: &mut World,
    ) {
        js::call(
            "appendLog",
            &[
                js::to_js(entries),
                js::to_js(text),
                JsValue::from_f64(effective_last as f64),
                JsValue::from_f64(effective_last as f64),
                JsValue::from_f64(effective_text_start as f64),
            ],
        );
    }
}

#[wasm_bindgen]
pub fn get_newest_log_messages() {
    let system = unsafe { &mut *SYSTEM };
    let world = &mut system.world();

    let last_log_entry: f64 = js::from_js(js::call0("logLastEntry")).unwrap_or(0.0);

    // TODO: ugly
    LogID::global_broadcast(world).get_after(
        last_log_entry as u32,
        500,
        LogUIID::local_first(world).into(),
        world,
    );
}

mod kay_auto;
pub use self::kay_auto::*;

pub fn setup(system: &mut ActorSystem) {
    system.register::<LogUI>();
    auto_setup(system);
}

pub fn spawn(world: &mut World) {
    LogUIID::spawn(world);
}
