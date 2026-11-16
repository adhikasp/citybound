use kay::World;
use wasm_bindgen::prelude::*;

use js;

pub trait FrameListener {
    fn on_frame(&mut self, world: &mut World);
}

pub fn flatten_vertices(vertices: &[::michelangelo::Vertex]) -> &[f32] {
    let new_len = vertices.len() * 3;
    unsafe { ::std::slice::from_raw_parts(vertices.as_ptr() as *const f32, new_len) }
}

pub fn flatten_instances(instances: &[::michelangelo::Instance]) -> &[f32] {
    let new_len = instances.len() * 8;
    unsafe { ::std::slice::from_raw_parts(instances.as_ptr() as *const f32, new_len) }
}

/// `{ vertices: Float32Array, indices: Uint16Array }`, the shape the WebGL side expects.
pub fn to_js_mesh(mesh: &::michelangelo::Mesh) -> JsValue {
    js::object(vec![
        ("vertices", js::f32_array(flatten_vertices(&mesh.vertices))),
        ("indices", js::u16_array(&mesh.indices)),
    ])
}

/// `[[groupId, mesh], ...]` - what `updatedVia("$add", ...)` expects.
pub fn updated_groups_to_js(group_changes: Vec<::michelangelo::GroupChange>) -> JsValue {
    js::array(
        group_changes
            .iter()
            .map(|change| {
                js::array(vec![
                    JsValue::from_f64(change.group_id as f64),
                    to_js_mesh(&change.new_group_mesh),
                ])
            })
            .collect(),
    )
}

mod kay_auto;
pub use self::kay_auto::*;
