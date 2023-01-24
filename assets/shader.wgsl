#import bevy_pbr::mesh_bindings
#import bevy_pbr::mesh_view_bindings

struct VertexOutput {
    @location(0) pos: vec3<f32>,
    @location(1) world_position: vec3<f32>,
    @location(2) cam_pos: vec3<f32>,
    @location(3) normal: vec3<f32>,
    @builtin(position) clip_position: vec4<f32>,
};

fn max_abs_e(v: vec3<f32>) -> f32 {
    return max(abs(v.x), max(abs(v.y), abs(v.z)));
}

let EPSILON = 0.0001;

@vertex
fn vertex(
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
) -> VertexOutput {
    let world_position = mesh.model * vec4(position, 1.0);
    let cam_pos = (mesh.inverse_transpose_model * vec4(view.world_position, 1.0)).xyz;
    var out: VertexOutput;

    let d = dot(normal, cam_pos - position);
    if d < 0.0 {
        out.pos = (cam_pos + 1.0) / 2.0;
    } else {
        out.pos = (position + 1.0) / 2.0;
    }

    out.world_position = world_position.xyz;
    out.cam_pos = cam_pos;
    out.clip_position = view.view_proj * world_position;
    out.normal = normal;
    return out;
}

@group(1) @binding(0)
var r_cells: texture_storage_3d<r8uint, read_write>;

struct Rule {
    size: u32,
    spawn_mode: u32,
    spawn_chance: f32,
    survival: u32,
    birth: u32,
    states: u32,
    neighbor_mode: u32,
    color_mode: u32,
    color0: vec4<f32>,
    color1: vec4<f32>,
};

@group(1) @binding(1)
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

let ERR_COLOR = vec3<f32>(1.0, 0.2, 0.6);

fn color(state: u32, p: vec3<f32>) -> vec3<f32> {
    switch i32(r_rule.color_mode) {
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

struct RayHit {
    fpos: vec3<f32>,
    vpos: vec3<f32>,
    norm: vec3<f32>,
    dist: f32,
    state: u32,
}

fn cast_ray(origin: vec3<f32>, dir: vec3<f32>, start_normal: vec3<f32>) -> RayHit {
    let step = sign(dir);
    let delta = min(step / dir, vec3(1.0 / EPSILON));
    var tmax = vec3(0.0);
    if step.x > 0.0 {
        tmax.x = delta.x * frac1(origin.x);
    } else {
        tmax.x = delta.x * frac0(origin.x);
    }
    if step.y > 0.0 {
        tmax.y = delta.y * frac1(origin.y);
    } else {
        tmax.y = delta.y * frac0(origin.y);
    }
    if step.z > 0.0 {
        tmax.z = delta.z * frac1(origin.z);
    } else {
        tmax.z = delta.z * frac0(origin.z);
    }
    var pos = floor(origin);
    var norm = start_normal;
    var dist = 0.0;

    loop {
        let state = textureLoad(r_cells, vec3<i32>(pos)).x;
        if state > u32(0) {
            var result: RayHit;
            result.fpos = origin + dir * dist;
            result.vpos = pos;
            result.norm = norm;
            result.dist = dist;
            result.state = state;

            return result;
        }
        if tmax.x < tmax.y {
            if tmax.x < tmax.z {
                pos.x = pos.x + step.x;
                norm = vec3(step.x, 0.0, 0.0);
                dist = tmax.x;
                if pos.x < 0.0 || pos.x >= f32(r_rule.size) {
                    break;
                }
                tmax.x = tmax.x + delta.x;
            } else {
                pos.z = pos.z + step.z;
                norm = vec3(0.0, 0.0, step.z);
                dist = tmax.z;
                if pos.z < 0.0 || pos.z >= f32(r_rule.size) {
                    break;
                }
                tmax.z = tmax.z + delta.z;
            }
        } else {
            if tmax.y < tmax.z {
                pos.y = pos.y + step.y;
                norm = vec3(0.0, step.y, 0.0);
                dist = tmax.y;
                if pos.y < 0.0 || pos.y >= f32(r_rule.size) {
                    break;
                }
                tmax.y = tmax.y + delta.y;
            } else {
                pos.z = pos.z + step.z;
                dist = tmax.z;
                norm = vec3(0.0, 0.0, step.z);
                if pos.z < 0.0 || pos.z >= f32(r_rule.size) {
                    break;
                }
                tmax.z = tmax.z + delta.z;
            }
        }
    }

    let dist = min(tmax.x, min(tmax.y, tmax.z));
    var result: RayHit;
    result.fpos = origin + dir * dist;
    result.vpos = pos;
    result.norm = norm;
    result.dist = dist;
    result.state = u32(0);

    return result;
}


fn trace_ray(origin: vec3<f32>, dir: vec3<f32>, start_normal: vec3<f32>) -> vec3<f32> {
    let result = cast_ray(origin, dir, start_normal);
    let light_dir = normalize(vec3<f32>(0.1, -1.0, 0.1));

    if result.state != u32(0) {
        let color = color(result.state, result.vpos);

        var light: f32;

        if cast_ray(result.fpos - light_dir * 0.01, -light_dir, -light_dir).state == u32(0) {
            light = (1.0 + dot(light_dir, result.norm)) / 2.0;
        } else {
            light = 0.0;
        }

        let ambient = 0.3 + 0.7 * (1.0 + dot(vec3(0.0, -1.0, 0.0), result.norm)) / 2.0;

        return color * (ambient * 0.3 + light * 0.7);
    } else {
        return vec3(0.0);
    }
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let dir = normalize(in.world_position - in.cam_pos);
    
    let fpos = in.pos.xyz * f32(r_rule.size);
    let p = normalize((in.pos.xyz - 0.5) * 2.0);
    let res = trace_ray(fpos, dir, -in.normal);

    return vec4(res, 1.0);
}