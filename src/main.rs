use nannou::prelude::*;
use nannou::wgpu;

struct Model {
    render: Render,
    compute: Compute,
    boids: Vec<Boid>,
}

struct Compute {
    boids_buffer: wgpu::Buffer,
    buffer_size: wgpu::BufferAddress,
    bind_group: wgpu::BindGroup,
    compute_pipeline: wgpu::ComputePipeline,
}

struct Render {
    bind_group: wgpu::BindGroup,
    render_pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct Boid {
    x: f32,
    y: f32,
    vx: f32,
    vy: f32,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct Vertex {
    position: [f32; 2],
}

fn main() {
    nannou::app(model).update(update).run();
}

const NUM_BOIDS: usize = 1000;

fn model(app: &App) -> Model {
    let w_id = app
        .new_window()
        .size(1000, 1000)
        .view(view)
        // .raw_event(raw_window_event)
        .build()
        .unwrap();

    // wgpu logical device
    let binding = app.window(w_id).unwrap();
    let device = binding.device();

    // Create the compute shader module
    let cs_module = device.create_shader_module(wgpu::include_wgsl!("shaders/compute.wgsl"));

    // Generate grid of boids
    let boids = generate_boids_grid(NUM_BOIDS, app.window_rect());

    // Create the buffer that will store the result of our compute operation.
    let buffer_size = (NUM_BOIDS * 2 as usize * std::mem::size_of::<f32>()) as wgpu::BufferAddress;
    let boids_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Compute"),
        size: buffer_size,
        usage: wgpu::BufferUsages::STORAGE
            | wgpu::BufferUsages::COPY_DST
            | wgpu::BufferUsages::COPY_SRC,
        mapped_at_creation: false,
    });

    let bind_group_layout = wgpu::BindGroupLayoutBuilder::new()
        .storage_buffer(wgpu::ShaderStages::COMPUTE, false, false)
        .build(device);

    let bind_group = wgpu::BindGroupBuilder::new()
        .buffer_bytes(
            &boids_buffer,
            0,
            Some(std::num::NonZeroU64::new(buffer_size).unwrap()),
        )
        .build(device, &bind_group_layout);

    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("nannou"),
        bind_group_layouts: &[&bind_group_layout],
        push_constant_ranges: &[],
    });

    let compute_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: Some("nannou"),
        layout: Some(&pipeline_layout),
        module: &cs_module,
        entry_point: "main",
    });

    let compute = Compute {
        boids_buffer,
        bind_group,
        compute_pipeline,
        buffer_size,
    };

    // RENDER PIPELINE

    let rn_module = device.create_shader_module(wgpu::include_wgsl!("shaders/render.wgsl"));

    let vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Vertex buffer"),
        size: (2 * NUM_BOIDS * std::mem::size_of::<f32>()) as u64,
        usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    let bind_group_layout = wgpu::BindGroupLayoutBuilder::new().build(device);
    let bind_group = wgpu::BindGroupBuilder::new().build(device, &bind_group_layout);
    let pipeline_layout = wgpu::create_pipeline_layout(device, None, &[&bind_group_layout], &[]);

    let render_pipeline = wgpu::RenderPipelineBuilder::from_layout(&pipeline_layout, &rn_module)
        .fragment_shader(&rn_module)
        .vertex_entry_point("vs_main")
        .fragment_entry_point("fs_main")
        .color_format(Frame::TEXTURE_FORMAT)
        .add_vertex_buffer::<Vertex>(&wgpu::vertex_attr_array![0 => Float32x2])
        .sample_count(binding.msaa_samples())
        .primitive_topology(wgpu::PrimitiveTopology::PointList)
        .build(device);

    let render = Render {
        bind_group,
        render_pipeline,
        vertex_buffer,
    };

    Model {
        render,
        compute,
        boids,
    }
}

fn update_boids(boids: &mut Vec<Boid>, dt: f32) {
    let turn_factor: f32 = 50.0;
    let visual_range: f32 = 100.0;
    let protected_range: f32 = 15.0;
    let centering_factor: f32 = 0.01;
    let avoid_factor: f32 = 0.05;
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
    update_boids(&mut model.boids, update.since_last.as_secs_f32());

    // let window = app.main_window();
    // let device = window.device();
    // let compute = &mut model.compute;

    // // The buffer to read the compute result
    // let read_buffer = device.create_buffer(&wgpu::BufferDescriptor {
    //     label: Some("read-boids"),
    //     size: compute.buffer_size,
    //     usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
    //     mapped_at_creation: false,
    // });

    // // The compute pass
    // let desc = wgpu::CommandEncoderDescriptor {
    //     label: Some("desc-compute"),
    // };
    // let mut encoder = device.create_command_encoder(&desc);
    // {
    //     let pass_desc = wgpu::ComputePassDescriptor {
    //         label: Some("boids-compute_pass"),
    //     };
    //     let mut cpass = encoder.begin_compute_pass(&pass_desc);
    //     cpass.set_pipeline(&compute.pipeline);
    //     cpass.set_bind_group(0, &compute.bind_group, &[]);
    //     cpass.dispatch_workgroups(1000 as u32, 1, 1);
    // }
    // encoder.copy_buffer_to_buffer(
    //     &compute.boids_buffer,
    //     0,
    //     &read_buffer,
    //     0,
    //     compute.buffer_size,
    // );

    // // Submit the compute pass to the device's queue.
    // window.queue().submit(Some(encoder.finish()));

    // // Spawn a future that reads the result of the compute pass.
    // let future = async move {
    //     let slice = read_buffer.slice(..);
    //     let (tx, rx) = futures::channel::oneshot::channel();
    //     slice.map_async(wgpu::MapMode::Read, |res| {
    //         tx.send(res).expect("The channel was closed");
    //     });
    //     if let Ok(_) = rx.await {
    //         let bytes = &slice.get_mapped_range()[..];
    //         println!("{}", std::str::from_utf8(bytes).unwrap());
    //         // let boids = {
    //         // let len = bytes.len() / std::mem::size_of::<f32>();
    //         // let ptr = bytes.as_ptr() as *const f32;
    //         // unsafe { std::slice::from_raw_parts(ptr, len) }
    //         // };
    //     }
    // };
    // pollster::block_on(future);
}

fn view(app: &App, model: &Model, frame: Frame) {
    // let draw = app.draw();
    // draw.background().color(WHITE);

    // draw.rect()
    //     .w(500.0)
    //     .h(500.0)
    //     .no_fill()
    //     .stroke_weight(5.0)
    //     .stroke(LIGHTGRAY);

    // model.boids.iter().for_each(|b| {
    //     draw.ellipse().x_y(b.x, b.y).radius(2.0).color(BLACK);
    // });

    // draw.to_frame(app, &frame).unwrap();
    // let device = app.main_window().device();

    let mut encoder = frame.command_encoder();

    let mut render_pass = wgpu::RenderPassBuilder::new()
        .color_attachment(frame.texture_view(), |color| color)
        .begin(&mut encoder);

    render_pass.set_bind_group(0, &model.render.bind_group, &[]);
    render_pass.set_pipeline(&model.render.render_pipeline);
    render_pass.set_vertex_buffer(0, model.render.vertex_buffer.slice(..));

    // Convert the vector of Boids to a slice of Vertex
    let mut vertices = Vec::with_capacity(NUM_BOIDS);
    for boid in &model.boids {
        vertices.push(Vertex {
            position: [boid.x / 500.0, boid.y / 500.0],
        });
    }

    let vertices_slice: &[Vertex] = &vertices[..NUM_BOIDS];
    let vertex_bytes: &[u8] = unsafe { wgpu::bytes::from_slice(&vertices_slice) };

    app.main_window()
        .queue()
        .write_buffer(&model.render.vertex_buffer, 0, &vertex_bytes);

    render_pass.draw(0..NUM_BOIDS as u32, 0..1);
}
