use kay::World;
use wasm_bindgen::prelude::*;

use js;

pub trait FrameListener {
    fn on_frame(&mut self, world: &mut World);
}

/// Flatten a mesh's vertices to `[x, y, z, x, y, z, ...]`.
///
/// Field by field rather than by reinterpreting the slice as raw `f32`s:
/// michelangelo's types are `repr(Rust)`, so the compiler is free to reorder
/// their fields, and it does (`flatten_instances` below is the case that bit).
pub fn flatten_vertices(vertices: &[::michelangelo::Vertex]) -> Vec<f32> {
    let mut flat = Vec::with_capacity(vertices.len() * 3);
    for vertex in vertices {
        flat.extend_from_slice(&vertex.position);
    }
    flat
}

/// Flatten instances to the layout Monet's shader expects:
/// `[position.x, position.y, position.z, direction.x, direction.y, r, g, b]`.
///
/// This used to be `from_raw_parts(instances.as_ptr() as *const f32, len * 8)`,
/// which assumed the fields of `michelangelo::Instance` sat in memory in the
/// order they were declared. `Instance` is `repr(Rust)`, so that is not
/// guaranteed: rustc 1.43 (the 2020 toolchain) happened to keep
/// position/direction/colour, while rustc 1.102 lays it out as
/// direction/position/colour. The raw cast then produced a valid-looking
/// buffer that the shader read wrongly - every tree got a "direction" of a
/// world coordinate and the 3D world drew nothing at all.
pub fn flatten_instances(instances: &[::michelangelo::Instance]) -> Vec<f32> {
    let mut flat = Vec::with_capacity(instances.len() * 8);
    for instance in instances {
        flat.extend_from_slice(&instance.instance_position);
        flat.extend_from_slice(&instance.instance_direction);
        flat.extend_from_slice(&instance.instance_color);
    }
    flat
}

/// `{ vertices: Float32Array, indices: Uint16Array }`, the shape the WebGL side expects.
pub fn to_js_mesh(mesh: &::michelangelo::Mesh) -> JsValue {
    js::object(vec![
        ("vertices", js::f32_array(&flatten_vertices(&mesh.vertices))),
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
