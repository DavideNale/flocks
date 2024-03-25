struct Boid {
    pos: vec2f,
    vel: vec2f,
}

@group(0) @binding(0) var<storage, read_write> boids: array<Boid>;
@group(0) @binding(1) var<storage, read_write> output: array<Boid>;

const turn_factor: f32 = 5.0;
const visual_range: f32 = 100.0;
const protected_range: f32 = 15.0;
const centering_factor: f32 = 0.01;
const avoid_factor: f32 = 0.05;
const matching_factor: f32 = 0.05;
const speed_max: f32 = 200.0;
const speed_min: f32 = 50.0;


@compute
@workgroup_size(256,1,1)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    // let index = length(global_id);
    let index = global_id.x + global_id.y + global_id.z;

    if index >= arrayLength(&boids){
        return;
    }

    var curr_p = boids[index].pos;
    var curr_v = boids[index].vel;
    var new_p = vec2f(0.0);
    var new_v = vec2f(0.0);

    var pos_cls = vec2f(0.0);
    var vel_avg = vec2f(0.0);
    var pos_avg = vec2f(0.0);
    var neighboring_boids: f32 = 0.0;

    for (var i : u32 = 0u; i < arrayLength(&boids); i = i + 1u) {
        if(i == index){
            continue;
        }

        let other_p: vec2f = boids[i].pos;
        let other_v: vec2f = boids[i].vel;

        let d: vec2f = curr_p - other_p;
        let dist = length(d);
        
        if dist < visual_range {
            if dist < protected_range {
                pos_cls += d;
                // curr_v += d * avoid_factor;
            }
            vel_avg += other_v;
            pos_avg += other_p;
            neighboring_boids += 1.0;            
        }

    }

    if neighboring_boids > 0.0 {
        new_v += (vel_avg / neighboring_boids - curr_v) * matching_factor;
        new_v += (pos_avg / neighboring_boids - curr_p) * centering_factor;
    }

    new_v += pos_cls * avoid_factor;

    if curr_p.y > 250.0 {
        new_v.y -= turn_factor;
    }
    if curr_p.y < -250.0 {
        new_v.y += turn_factor;
    }
    if curr_p.x > 250.0 {
        new_v.x -= turn_factor;
    }
    if curr_p.x < -250.0{
        new_v.x += turn_factor;
    }

    new_v += curr_v;

    let speed = length(new_v);
    if speed > speed_max {
        new_v = new_v / speed * speed_max;
    }
    if speed < speed_min {
        new_v = new_v / speed * speed_min;
    }

    curr_p += new_v * 0.005;
    // curr_p += curr_v * 16.0;

    output[index] = Boid(curr_p, new_v);
}
