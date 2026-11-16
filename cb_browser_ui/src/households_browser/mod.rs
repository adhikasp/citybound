use kay::{World, ActorSystem, TypedID};
use wasm_bindgen::prelude::*;

use js;
use SYSTEM;

#[wasm_bindgen]
pub fn get_household_info(household_id: JsValue) {
    let system = unsafe { &mut *SYSTEM };
    let world = &mut system.world();
    if let Some(household_id) = js::from_js::<::economy::households::HouseholdID>(household_id) {
        household_id.get_ui_info(BrowserHouseholdUIID::local_first(world).into(), world);
    }
}

#[derive(Compact, Clone)]
pub struct BrowserHouseholdUI {
    id: BrowserHouseholdUIID,
}

impl BrowserHouseholdUI {
    pub fn spawn(id: BrowserHouseholdUIID, _: &mut World) -> BrowserHouseholdUI {
        BrowserHouseholdUI { id }
    }
}

use economy::households::ui::{HouseholdUI, HouseholdUIID};

impl HouseholdUI for BrowserHouseholdUI {
    fn on_household_ui_info(
        &mut self,
        id: ::economy::households::HouseholdID,
        core: &::economy::households::HouseholdCore,
        _world: &mut World,
    ) {
        // TODO: horrible workaround for Compact mis-serialising the best_offer COption (probably
        // because it's inside a CDict)
        let decision_state_workaround = match core.decision_state {
            ::economy::households::DecisionState::Choosing(m, i, ref rsrc, _) => {
                ::economy::households::DecisionState::Choosing(
                    m,
                    i,
                    rsrc.clone(),
                    ::compact::CDict::new(),
                )
            }
            ref other => other.clone(),
        };

        js::call(
            "householdInfo",
            &[
                js::to_js(&id),
                js::object(vec![
                    ("resources", js::to_js(&core.resources)),
                    ("member_resources", js::to_js(&core.member_resources)),
                    ("member_tasks", js::to_js(&core.member_tasks)),
                    ("decision_state", js::to_js(&decision_state_workaround)),
                    ("used_offers", js::to_js(&core.used_offers)),
                    ("member_used_offers", js::to_js(&core.member_used_offers)),
                    ("provided_offers", js::to_js(&core.provided_offers)),
                ]),
            ],
        );
    }
}

mod kay_auto;
pub use self::kay_auto::*;

pub fn setup(system: &mut ActorSystem) {
    system.register::<BrowserHouseholdUI>();
    auto_setup(system);
}

pub fn spawn(world: &mut World) {
    BrowserHouseholdUIID::spawn(world);
}
