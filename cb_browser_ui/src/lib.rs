#![recursion_limit = "256"]
// kay_codegen emits #![rustfmt::skip] into the generated kay_auto.rs files
#![feature(custom_inner_attributes)]


extern crate serde;

#[macro_use]
extern crate serde_derive;

#[macro_use]
extern crate compact_macros;

extern crate wasm_bindgen;
extern crate js_sys;
extern crate web_sys;

extern crate kay;
use kay::{ActorSystem, TypedID};

extern crate cb_simulation;
use cb_simulation::*;

extern crate cb_util;
extern crate cb_time;
extern crate cb_planning;

use std::panic;

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

pub mod js;

pub mod planning_browser;
pub mod debug;
pub mod time_browser;
pub mod households_browser;
pub mod transport_browser;
pub mod land_use_browser;
pub mod vegetation_browser;
pub mod browser_utils;

// TODO: not thread safe for now
static mut SYSTEM: *mut ActorSystem = 0 as *mut ActorSystem;

#[allow(static_mut_refs)]
fn system() -> &'static mut ActorSystem {
    unsafe { &mut *SYSTEM }
}

#[wasm_bindgen]
pub fn start() {
    panic::set_hook(Box::new(|info| {
        js::call("panic", &[JsValue::from_str(&info.to_string())]);
    }));

    js::console_log("Before setup");

    let server_host = js::call0("hostname").as_string().unwrap_or_default();

    let network_settings_js = js::call0("networkSettings");
    let network_settings: std::collections::HashMap<String, f64> =
        js::from_js(network_settings_js).unwrap_or_default();

    let mut system = kay::ActorSystem::new(
        kay::Networking::new(
            1,
            vec![format!("{}:{}", server_host, 9999), "ws-client".to_owned()],
            network_settings["batchMessageBytes"] as usize,
            network_settings["acceptableTurnDistance"] as usize,
            network_settings["skipTurnsPerTurnAhead"] as usize,
        ),
        kay::Tuning::default(),
    );

    setup_common(&mut system);
    debug::setup(&mut system);
    browser_utils::auto_setup(&mut system);
    planning_browser::setup(&mut system);
    transport_browser::setup(&mut system);
    time_browser::setup(&mut system);
    land_use_browser::setup(&mut system);
    households_browser::setup(&mut system);
    vegetation_browser::setup(&mut system);

    js::call(
        "setTypeIdMapping",
        &[js::to_js(&system.get_actor_type_id_to_name_mapping())],
    );

    system.networking_connect();

    debug::spawn(&mut system.world());
    planning_browser::spawn(&mut system.world());
    transport_browser::spawn(&mut system.world());
    time_browser::spawn(&mut system.world());
    land_use_browser::spawn(&mut system.world());
    households_browser::spawn(&mut system.world());
    vegetation_browser::spawn(&mut system.world());

    system.process_all_messages();

    js::console_log("After setup");

    let mut main_loop = MainLoop { skip_turns: 0 };

    unsafe { SYSTEM = Box::into_raw(Box::new(system)) };

    main_loop.frame();
}

#[derive(Copy, Clone)]
struct MainLoop {
    skip_turns: usize,
}

impl MainLoop {
    #[allow(static_mut_refs)]
    fn frame(&mut self) {
        let system = unsafe { &mut *SYSTEM };
        let world = &mut system.world();

        system.networking_send_and_receive();

        if self.skip_turns == 0 {
            system.process_all_messages();

            browser_utils::FrameListenerID::local_broadcast(world).on_frame(world);
            system.process_all_messages();

            system.networking_send_and_receive();
            system.process_all_messages();
        }

        js::call(
            "systemStats",
            &[
                js::to_js(&system.networking_debug_all_n_turns()),
                js::to_js(&system.get_queue_lengths()),
                js::to_js(&system.get_message_statistics()),
            ],
        );
        js::call0("onFrame");

        if self.skip_turns == 0 {
            system.reset_message_statistics();
        }

        let mut next = *self;

        if self.skip_turns > 0 {
            next.skip_turns -= 1;
        } else {
            let maybe_should_skip = system.networking_finish_turn();
            if let Some(should_skip) = maybe_should_skip {
                next.skip_turns = should_skip
            }
        }

        request_animation_frame(next);
    }
}

fn request_animation_frame(main_loop: MainLoop) {
    let callback = Closure::once(move || {
        let mut main_loop = main_loop;
        main_loop.frame();
    });

    js::window()
        .request_animation_frame(callback.as_ref().unchecked_ref())
        .expect("requestAnimationFrame failed");

    callback.forget();
}

#[wasm_bindgen]
pub fn point_in_area(point: JsValue, area: JsValue) -> bool {
    use ::descartes::PointContainer;

    match (
        js::from_js::<descartes::P2>(point),
        js::from_js::<descartes::Area>(area),
    ) {
        (Some(point), Some(area)) => area.contains(point),
        _ => false,
    }
}

#[wasm_bindgen]
pub fn point_close_to_path(
    point: JsValue,
    path: JsValue,
    max_distance_right: JsValue,
    max_distance_left: JsValue,
) -> JsValue {
    use ::descartes::WithUniqueOrthogonal;

    let (point, path, max_distance_right, max_distance_left): (
        descartes::P2,
        descartes::LinePath,
        descartes::N,
        descartes::N,
    ) = match (
        js::from_js(point),
        js::from_js(path),
        js::from_js(max_distance_right),
        js::from_js(max_distance_left),
    ) {
        (Some(point), Some(path), Some(max_distance_right), Some(max_distance_left)) => (
            point,
            path,
            max_distance_right,
            max_distance_left,
        ),
        _ => return JsValue::NULL,
    };

    let result = path
        .project(point)
        .and_then(|(distance, projected_point)| {
            let direction = path.direction_along(distance);
            let orth_distance =
                (point - projected_point).dot(&direction.orthogonal_right());
            if orth_distance >= -max_distance_left && orth_distance <= max_distance_right {
                Some((point, projected_point, direction))
            } else {
                None
            }
        });

    js::to_js(&result)
}
