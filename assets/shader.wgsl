#import bevy_pbr::mesh_view_bind_group
#import bevy_pbr::mesh_struct

struct VertexOutput {
    [[location(0)]] pos: vec3<f32>;
    [[location(1)]] world_position: vec3<f32>;
    [[location(2)]] cam_pos: vec3<f32>;
    [[builtin(position)]] clip_position: vec4<f32>;
};

[[group(2), binding(0)]]
var<uniform> mesh: Mesh;

fn max_abs_e(v: vec3<f32>) -> f32 {
    return max(abs(v.x), max(abs(v.y), abs(v.z)));
}

let EPSILON = 0.0001;

[[stage(vertex)]]
fn vertex(
    [[location(0)]] position: vec3<f32>,
) -> VertexOutput {
    let world_position = mesh.model * vec4<f32>(position, 1.0);
    let cam_pos = (mesh.inverse_transpose_model * vec4<f32>(view.world_position, 1.0)).xyz;
    var out: VertexOutput;
    
    if (-1.0 < cam_pos.x && cam_pos.x < 1.0 && -1.0 < cam_pos.y && cam_pos.y < 1.0 && -1.0 < cam_pos.z && cam_pos.z < 1.0) {
        out.pos = (cam_pos + 1.0) / 2.0;
    } else {
        out.pos = (position + 1.0) / 2.0;
    }

    out.world_position = world_position.xyz;
    out.cam_pos = cam_pos;
    out.clip_position = view.view_proj * world_position;
    return out;
}

[[group(1), binding(0)]]
var r_cells: texture_3d<u32>;
fn get(pos: vec3<i32>, offset_x: i32, offset_y: i32, offset_z: i32) -> i32 {
    let value: vec4<u32> = textureLoad(r_cells, pos + vec3<i32>(offset_x, offset_y, offset_z), 0);
    return i32(value.x);
}

struct Rule {
    size: u32;
    spawn_chance: f32;
    survival: u32;
    birth: u32;
    states: u32;
    neighbor_mode: u32;
    color_mode: u32;
    color0: vec4<f32>;
    color1: vec4<f32>;
};
[[group(1), binding(1)]]
var<uniform> r_rule: Rule;

/// MOVE THIS TO IMPORT
fn hash(value: u32) -> u32 {
    var state = value;
    state = state ^ 2747636419u;
    state = state * 2654435769u;
    state = state ^ state >> 16u;
    state = state * 2654435769u;
    state = state ^ state >> 16u;
    state = state * 2654435769u;
    return state;
}
fn random_float(value: u32) -> f32 {
    return f32(hash(value)) / 4294967295.0;
}
/// END

fn is_outside(pos: vec3<f32>) -> bool {
    return pos.x < -EPSILON || pos.x > f32(r_rule.size) + EPSILON ||
            pos.y < -EPSILON || pos.y > f32(r_rule.size) + EPSILON ||
            pos.z < -EPSILON || pos.z > f32(r_rule.size) + EPSILON;
}

let ERR_COLOR = vec3<f32>(1.0, 0.2, 0.6);

fn color(state: u32, p: vec3<f32>) -> vec3<f32> {
    switch (r_rule.color_mode) {
        case 0: {
            return r_rule.color0.xyz;
        }
        case 1: {
            let t = f32(state) / f32(r_rule.states);
            return r_rule.color0.xyz * (1.0 - t) + r_rule.color1.xyz * t;
        }
        case 2: {
            let t = length(p - f32(r_rule.size) / 2.0) / f32(r_rule.size);
            return r_rule.color0.xyz * (1.0 - t) + r_rule.color1.xyz * t;
        }
        case 3: {
            // TODO: impl neighbor coloring
            return ERR_COLOR;
        }
        default: {
            return r_rule.color0.xyz;
        }
    }
}

fn frac0(v: f32) -> f32 {
    return (v - floor(v));
}
fn frac1(v: f32) -> f32 {
    return (1.0 - v + floor(v));
}

fn cast_ray(origin: vec3<f32>, dir: vec3<f32>) -> vec3<f32> {
    let clear = vec3<f32>(0.0, 0.0, 0.0);

    
    let step = sign(dir);
    let delta = min(step / dir, vec3<f32>(1.0 / EPSILON));
    var tmax = vec3<f32>(0.0);
    if (step.x > 0.0) {
        tmax.x = delta.x * frac1(origin.x);
    } else {
        tmax.x = delta.x * frac0(origin.x);
    }
    if (step.y > 0.0) {
        tmax.y = delta.y * frac1(origin.y);
    } else {
        tmax.y = delta.y * frac0(origin.y);
    }
    if (step.z > 0.0) {
        tmax.z = delta.z * frac1(origin.z);
    } else {
        tmax.z = delta.z * frac0(origin.z);
    }
    var pos = floor(origin);

    loop {
        let state = textureLoad(r_cells, vec3<i32>(pos), 0).x;
        if (state > u32(0)) {
            return color(state, pos);
        }
        if (tmax.x < tmax.y) {
            if (tmax.x < tmax.z) {
                pos.x = pos.x + step.x;
                if (pos.x < 0.0 || pos.x >= f32(r_rule.size)) {
                    break;
                }
                tmax.x = tmax.x + delta.x;
            } else {
                pos.z = pos.z + step.z;
                if (pos.z < 0.0 || pos.z >= f32(r_rule.size)) {
                    break;
                }
                tmax.z = tmax.z + delta.z;
            }
        } else {
            if (tmax.y < tmax.z) {
                pos.y = pos.y + step.y;
                if (pos.y < 0.0 || pos.y >= f32(r_rule.size)) {
                    break;
                }
                tmax.y = tmax.y + delta.y;
            } else {
                pos.z = pos.z + step.z;
                if (pos.z < 0.0 || pos.z >= f32(r_rule.size)) {
                    break;
                }
                tmax.z = tmax.z + delta.z;
            }
        }
    }
    return clear;
    // var pos = origin;
    // loop {
    //     let ipos = vec3<i32>(pos);
    //     let state = textureLoad(r_cells, ipos, 0).x;
    //     if (state > u32(0)) {
    //         return color(state); // + (random_float(u32(ipos.z) * r_rule.size * r_rule.size + u32(ipos.y) * r_rule.size + u32(ipos.x)) - 0.5) * 0.05;
    //     }
    //     pos = pos + dir;
    //     if (is_outside(pos)) {
    //         break;
    //     }
    // }
    // return clear;
}

[[stage(fragment)]]
fn fragment(in: VertexOutput) -> [[location(0)]] vec4<f32> {
    let dir = normalize(in.world_position - in.cam_pos);
    
    let fpos = in.pos.xyz * f32(r_rule.size);
    let p = normalize((in.pos.xyz - 0.5) * 2.0);
    let res = cast_ray(fpos, dir);

    return vec4<f32>(res, 1.0);
}