use kay::{World, ActorSystem, TypedID};
use compact::{CVec, CHashMap, COption};
use wasm_bindgen::prelude::*;

use js;
use browser_utils::{to_js_mesh, flatten_instances};
use SYSTEM;
use cb_util::config_manager::{Name, ConfigUser, ConfigUserID};
use ::land_use::buildings::{BuildingID, BuildingStyle};
use ::land_use::buildings::architecture::{build_building};
use ::land_use::buildings::architecture::language::ArchitectureRule;
use ::land_use::buildings::architecture::materials_and_props::{ALL_MATERIALS, ALL_PROP_TYPES};
use ::land_use::zone_planning::Lot;
use ::economy::households::HouseholdID;

#[wasm_bindgen]
pub fn get_building_info(building_id: JsValue) {
    let system = unsafe { &mut *SYSTEM };
    let world = &mut system.world();
    if let Some(building_id) = js::from_js::<BuildingID>(building_id) {
        building_id.get_ui_info(BrowserLandUseUIID::local_first(world).into(), world);
    }
}

#[derive(Compact, Clone)]
pub struct BrowserLandUseUI {
    id: BrowserLandUseUIID,
    architecture_rules: CHashMap<Name, ArchitectureRule>,
}

impl BrowserLandUseUI {
    pub fn spawn(id: BrowserLandUseUIID, world: &mut World) -> BrowserLandUseUI {
        {
            BuildingID::global_broadcast(world).get_render_info(id.into(), world);
        }

        let ui = BrowserLandUseUI {
            id,
            architecture_rules: CHashMap::new(),
        };
        ui.get_initial_config(world);
        ui
    }
}

impl ConfigUser<ArchitectureRule> for BrowserLandUseUI {
    fn local_cache(&mut self) -> &mut CHashMap<Name, ArchitectureRule> {
        &mut self.architecture_rules
    }

    fn on_config_change(
        &mut self,
        name: Name,
        maybe_value: &COption<ArchitectureRule>,
        world: &mut World,
    ) {
        // TODO: make this more specific
        BuildingID::global_broadcast(world).get_render_info(self.id.into(), world);

        self.apply_config_change(name, maybe_value, world);
    }
}

use land_use::ui::{LandUseUI, LandUseUIID};

impl LandUseUI for BrowserLandUseUI {
    fn on_building_constructed(
        &mut self,
        id: BuildingID,
        lot: &Lot,
        households: &CVec<HouseholdID>,
        style: BuildingStyle,
        world: &mut World,
    ) {
        let result = build_building(lot, style, &self.architecture_rules, households, world);
        match result {
            Ok(building_mesh) => {
                let material_meshes = building_mesh
                    .meshes
                    .into_iter()
                    .map(|(material, mesh)| {
                        js::array(vec![js::to_js(&material.to_string()), to_js_mesh(&mesh)])
                    })
                    .collect::<Vec<_>>();

                let prop_instances = building_mesh
                    .props
                    .into_iter()
                    .map(|(prop_type, instances)| {
                        js::array(vec![
                            js::to_js(&prop_type.to_string()),
                            js::f32_array(&flatten_instances(&instances)),
                        ])
                    })
                    .collect::<Vec<_>>();

                js::call(
                    "buildingConstructed",
                    &[
                        js::to_js(&id),
                        js::array(material_meshes),
                        js::array(prop_instances),
                        js::to_js(&lot.center_point()),
                        js::to_js(&lot.area),
                    ],
                );
            }
            Err(reason) => {
                js::console_error(&format!("Building geometry generation error: {:?}", reason));
            }
        };
    }

    fn on_building_destructed(&mut self, id: BuildingID, _world: &mut World) {
        js::call(
            "buildingDestructed",
            &[
                js::to_js(&id),
                js::array(
                    ALL_MATERIALS
                        .iter()
                        .map(|material| js::to_js(&material.to_string()))
                        .collect(),
                ),
                js::array(
                    ALL_PROP_TYPES
                        .iter()
                        .map(|prop_type| js::to_js(&prop_type.to_string()))
                        .collect(),
                ),
            ],
        );
    }

    fn on_building_ui_info(
        &mut self,
        _id: BuildingID,
        style: BuildingStyle,
        households: &CVec<HouseholdID>,
        _world: &mut World,
    ) {
        js::call(
            "inspectedBuilding",
            &[js::to_js(households), js::to_js(&style)],
        );
    }
}

mod kay_auto;
pub use self::kay_auto::*;

pub fn setup(system: &mut ActorSystem) {
    system.register::<BrowserLandUseUI>();
    auto_setup(system);
}

pub fn spawn(world: &mut World) {
    BrowserLandUseUIID::spawn(world);
}
