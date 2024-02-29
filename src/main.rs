use nannou::prelude::*;

fn main() {
    nannou::app(model).update(update).run();
}

struct Boid {
    x: f32,
    y: f32,
}

fn update_boids(boids: &mut Vec<Boid>, dt: f32) {
    for i in 0..boids.len() {
        let protected: f32 = 50.0;
        let mut close_dx: f32 = 0.0;
        let mut close_dy: f32 = 0.0;

        let x = boids[i].x;
        let y = boids[i].y;

        for other_boid in &mut *boids {
            let distance2 = (other_boid.x - x).powi(2) + (other_boid.y - y).powi(2);

            if distance2 != 0.0 && distance2 <= protected.powi(2) {
                close_dx += other_boid.x;
                close_dy += other_boid.y;
            }
        }

        boids[i].x -= close_dx * dt;
        boids[i].y -= close_dy * dt;
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
            })
        })
        .collect();
    boids
}

fn update(_app: &App, model: &mut Model, update: Update) {
    update_boids(&mut model.boids, update.since_last.as_secs_f32());
}

fn view(app: &App, model: &Model, frame: Frame) {
    let draw = app.draw();
    draw.background().color(WHITE);

    model.boids.iter().for_each(|b| {
        draw.ellipse().x_y(b.x, b.y).radius(2.0).color(BLACK);
    });

    draw.to_frame(app, &frame).unwrap();
}
