use nannou::prelude::*;

fn main() {
    nannou::app(model).update(update).run();
}

struct Boid {
    x: f32,
    y: f32,
    vx: f32,
    vy: f32,
}

fn update_boids(boids: &mut Vec<Boid>, dt: f32) {
    let turn_factor: f32 = 10.0;
    let visual_range: f32 = 100.0;
    let protected_range: f32 = 15.0;
    let centering_factor: f32 = 0.005;
    let avoid_factor: f32 = 1.0;
    let matching_factor: f32 = 0.05;
    let speed_max: f32 = 200.0;
    let speed_min: f32 = 50.0;

    for i in 0..boids.len() {
        // Save i-th boid's transform
        let x = boids[i].x;
        let y = boids[i].y;
        let mut vx = boids[i].vx;
        let mut vy = boids[i].vy;

        // Zero accumulators
        let mut close_dx: f32 = 0.0;
        let mut close_dy: f32 = 0.0;

        let mut xvel_avg: f32 = 0.0;
        let mut yvel_avg: f32 = 0.0;
        let mut neighboring_boids: f32 = 0.0;

        let mut xpos_avg: f32 = 0.0;
        let mut ypos_avg: f32 = 0.0;

        // Iterate each boid
        for j in 0..boids.len() {
            if i == j {
                continue;
            }

            let other_x: f32 = boids[j].x;
            let other_y: f32 = boids[j].y;
            let other_vx: f32 = boids[j].vx;
            let other_vy: f32 = boids[j].vy;

            let dx = x - other_x;
            let dy = y - other_y;
            let dist = (dx * dx + dy * dy).sqrt();

            if dist < visual_range {
                // Separation
                if dist < protected_range {
                    close_dx += dx;
                    close_dy += dy;
                }

                // Alignment
                xvel_avg += other_vx;
                yvel_avg += other_vy;

                // Cohesion
                xpos_avg += other_x;
                ypos_avg += other_y;

                neighboring_boids += 1.0;
            }
        }
        // Update i-th boid's transform

        if neighboring_boids > 0.0 {
            xvel_avg = xvel_avg / neighboring_boids;
            yvel_avg = yvel_avg / neighboring_boids;
            xpos_avg = xpos_avg / neighboring_boids;
            ypos_avg = ypos_avg / neighboring_boids;

            vx += (xpos_avg - x) * centering_factor + (xvel_avg - vx) * matching_factor;
            vy += (ypos_avg - y) * centering_factor + (yvel_avg - vy) * matching_factor;
        }

        vx += close_dx * avoid_factor;
        vy += close_dy * avoid_factor;

        // Edge avoidance
        if y > 250.0 {
            vy -= turn_factor;
        }
        if y < -250.0 {
            vy += turn_factor;
        }
        if x > 250.0 {
            vx -= turn_factor;
        }
        if x < -250.0 {
            vx += turn_factor;
        }

        // Speed adjustment
        let speed = (vx * vx + vy * vy).sqrt();
        if speed < speed_min {
            vx = (vx / speed) * speed_min;
            vy = (vy / speed) * speed_min;
        }
        if speed > speed_max {
            vx = (vx / speed) * speed_max;
            vy = (vy / speed) * speed_max;
        }

        // println!("{} {}", vx, vy);
        boids[i].vx = vx;
        boids[i].vy = vy;

        boids[i].x += vx * dt;
        boids[i].y += vy * dt;
    }
}

struct Model {
    boids: Vec<Boid>,
    num_boids: usize,
}

fn model(app: &App) -> Model {
    app.new_window()
        .size(1000, 1000)
        .view(view)
        .build()
        .unwrap();

    let num_boids = 1000;
    let boids = generate_boids_grid(num_boids, app.window_rect());

    Model { boids, num_boids }
}

fn generate_boids_grid(num_boids: usize, _rect: Rect) -> Vec<Boid> {
    // let w = rect.w();
    // let h = rect.h();

    let side = (num_boids as f32).sqrt().ceil() as usize;
    let delta: f32 = 10.0;
    let boids: Vec<Boid> = (0..side)
        .flat_map(|i| {
            (0..side).map(move |j| Boid {
                x: i as f32 * delta - (side as f32 * delta) / 2.0,
                y: j as f32 * delta - (side as f32 * delta) / 2.0,
                vx: 0.0,
                vy: 0.0,
            })
        })
        .collect();

    boids
}

fn update(_app: &App, model: &mut Model, update: Update) {
    println!("{}", 1.0 / update.since_last.as_secs_f32());
    update_boids(&mut model.boids, update.since_last.as_secs_f32());
}

fn view(app: &App, model: &Model, frame: Frame) {
    let draw = app.draw();
    draw.background().color(WHITE);

    draw.rect()
        .w(500.0)
        .h(500.0)
        .no_fill()
        .stroke_weight(5.0)
        .stroke(LIGHTGRAY);

    model.boids.iter().for_each(|b| {
        draw.ellipse().x_y(b.x, b.y).radius(2.0).color(BLACK);
    });

    draw.to_frame(app, &frame).unwrap();
}
