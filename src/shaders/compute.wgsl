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
const avoid_factor: f32 = 0.5;
const matching_factor: f32 = 0.05;
const speed_max: f32 = 200.0;
const speed_min: f32 = 50.0;


@compute
@workgroup_size(16,16,1)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let index = global_id.x + global_id.y + global_id.z;
    let len : u32 = arrayLength(&boids);

    if index >= len {
        return;
    }

    var curr_p = boids[index].pos;
    var curr_v = boids[index].vel;

    var pos_cls = vec2f(0.0);
    var vel_avg = vec2f(0.0);
    var pos_avg = vec2f(0.0);
    var neighboring_boids: f32 = 0.0;

    for (var i : u32 = 0u; i < len; i = i + 1u) {
        if(i == index){
            continue;
        }

        let other_p: vec2f = boids[i].pos;
        let other_v: vec2f = boids[i].vel;

        let d: vec2f = curr_p - other_p;
        let dist = sqrt(d.x*d.x + d.y*d.y);
        
        if dist < visual_range {
            if dist < protected_range {
                curr_v += d * avoid_factor;
            }
            vel_avg += other_v;
            pos_avg += other_p;
            neighboring_boids += 1.0;            
        }

    }

    if curr_p.y > 250.0 {
        curr_v.y -= turn_factor;
    }
    if curr_p.y < -250.0 {
        curr_v.y += turn_factor;
    }
    if curr_p.x > 250.0 {
        curr_v.x -= turn_factor;
    }
    if curr_p.x < -250.0{
        curr_v.x += turn_factor;
    }

    if neighboring_boids > 0.0 {
        curr_v += (vel_avg/neighboring_boids - curr_v) * matching_factor;
        curr_v += (pos_avg/neighboring_boids - curr_p) * centering_factor;
    }

    let speed = sqrt(curr_v.x*curr_v.x + curr_v.y*curr_v.y);
    if speed > speed_max {
        curr_v = curr_v/speed_max;
    }
    if speed < speed_min {
        curr_v = curr_v/speed_min;
    }

    curr_p += curr_v * 10.0;

    output[index] = Boid(curr_p, curr_v);
}
