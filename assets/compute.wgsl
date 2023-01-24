
@group(0) @binding(0)
var r_cells: texture_storage_3d<r8uint, read_write>;

fn get_cell(pos: vec3<i32>, offset_x: i32, offset_y: i32, offset_z: i32) -> i32 {
    let value: vec4<u32> = textureLoad(r_cells, pos + vec3(offset_x, offset_y, offset_z));
    return i32(value.x);
}

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

@group(0) @binding(1)
var<uniform> r_rule: Rule;

fn is_alive(value: i32) -> i32 {
    return value / i32(r_rule.states);
}

fn should_survive(num_neighbours: i32) -> bool {
    return ((r_rule.survival >> u32(num_neighbours)) & u32(1)) != u32(0);
}

fn should_birth(num_neighbours: i32) -> bool {
    return ((r_rule.birth >> u32(num_neighbours)) & u32(1)) != u32(0);
}

fn count_alive(pos: vec3<i32>) -> i32 {
    switch i32(r_rule.neighbor_mode) {
        case 0: {
            return is_alive(get_cell(pos, -1, -1, -1)) + 
                   is_alive(get_cell(pos, -1, -1,  0)) + 
                   is_alive(get_cell(pos, -1, -1,  1)) + 
                   is_alive(get_cell(pos, -1,  0, -1)) + 
                   is_alive(get_cell(pos, -1,  0,  0)) + 
                   is_alive(get_cell(pos, -1,  0,  1)) + 
                   is_alive(get_cell(pos, -1,  1, -1)) + 
                   is_alive(get_cell(pos, -1,  1,  0)) + 
                   is_alive(get_cell(pos, -1,  1,  1)) + 
       
                   is_alive(get_cell(pos,  0, -1, -1)) + 
                   is_alive(get_cell(pos,  0, -1,  0)) + 
                   is_alive(get_cell(pos,  0, -1,  1)) + 
                   is_alive(get_cell(pos,  0,  0, -1)) + 
                   //is_alive(get_cell(pos,  0,  0,  0)) + Don't count yourself
                   is_alive(get_cell(pos,  0,  0,  1)) + 
                   is_alive(get_cell(pos,  0,  1, -1)) + 
                   is_alive(get_cell(pos,  0,  1,  0)) + 
                   is_alive(get_cell(pos,  0,  1,  1)) + 
       
                   is_alive(get_cell(pos,  1, -1, -1)) + 
                   is_alive(get_cell(pos,  1, -1,  0)) + 
                   is_alive(get_cell(pos,  1, -1,  1)) + 
                   is_alive(get_cell(pos,  1,  0, -1)) + 
                   is_alive(get_cell(pos,  1,  0,  0)) + 
                   is_alive(get_cell(pos,  1,  0,  1)) + 
                   is_alive(get_cell(pos,  1,  1, -1)) + 
                   is_alive(get_cell(pos,  1,  1,  0)) + 
                   is_alive(get_cell(pos,  1,  1,  1));
        }
        case 1: {
            return is_alive(get_cell(pos,  0,  0, -1)) + 
                   is_alive(get_cell(pos,  0,  0,  1)) + 
                   is_alive(get_cell(pos,  0, -1,  0)) + 
                   is_alive(get_cell(pos,  0,  1,  0)) + 
                   is_alive(get_cell(pos, -1,  0,  0)) + 
                   is_alive(get_cell(pos,  1,  0,  0));
        }
        default: {
            return 0;
        }
    }
}

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

@compute @workgroup_size(9, 9, 9)
fn init(@builtin(global_invocation_id) pos: vec3<u32>) {
    var alive = false;
    switch i32(r_rule.spawn_mode) {
        // Random
        case 0: {
            let random_number = random_float(pos.z * r_rule.size * r_rule.size + pos.y * r_rule.size + pos.x);
            alive = random_number > r_rule.spawn_chance;
        }
        // Menger Sponge
        case 1: {
            let size = r_rule.size;
            var i = u32(3);
            loop {
                let s = size / i;
                if size - s * i != u32(0) {
                    alive = true;
                    break;
                }
                let p = abs(vec3<i32>((pos / s) - (pos / s) / u32(3) * u32(3)) - vec3(1));
                if p.x + p.y + p.z <= 1 {
                    break;
                }
                i = i * u32(3);
            }
        }
        default: {}
    }
    
    
    textureStore(r_cells, vec3<i32>(pos), vec4<u32>(u32(alive) * u32(r_rule.states)));
}

@compute @workgroup_size(9, 9, 9)
fn update(@builtin(global_invocation_id) invocation_id: vec3<u32>) {
    let pos = vec3<i32>(invocation_id);
    var cur = get_cell(pos, 0, 0, 0);

    let alive = count_alive(pos);

    if is_alive(cur) == 1 {
        if !should_survive(alive) {
            cur = cur - 1;
        }
    } else if cur == 0 {
        if should_birth(alive) {
            cur = i32(r_rule.states);
        }
    } else {
        cur = cur - 1;
    }

    let alive = count_alive(pos);

    let res = u32(cur);
    storageBarrier();
    textureStore(r_cells, pos, vec4<u32>(res));
}