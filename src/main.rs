use bytemuck::{Pod, Zeroable};
use futures::sink::Buffer;
use nannou::prelude::*;
use nannou::wgpu;
use nannou::wgpu::Device;

struct Model {
    render: Render,
    compute: Compute,
    boids: Vec<Boid>,
}

struct Compute {
    buffer_size: u64,
    boids_buffer: wgpu::Buffer,
    output_buffer: wgpu::Buffer,
    bind_group_layout: wgpu::BindGroupLayout,
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
unsafe impl Pod for Boid {}
unsafe impl Zeroable for Boid {}

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
        .build()
        .unwrap();

    // wgpu logical device
    let binding = app.window(w_id).unwrap();
    let device = binding.device();

    // COMPUTE PIPELINE

    let cs_module = device.create_shader_module(wgpu::include_wgsl!("shaders/compute.wgsl"));

    let buffer_size: u64 = (NUM_BOIDS * 4 * std::mem::size_of::<f32>()) as u64;

    let boids = generate_boids_grid(NUM_BOIDS, app.window_rect());
    let boids_bytes: &[u8] = unsafe { wgpu::bytes::from_slice(&boids[..NUM_BOIDS]) };

    let boids_buffer = device.create_buffer_init(&wgpu::BufferInitDescriptor {
        label: Some("Compute input buffer"),
        contents: &boids_bytes,
        usage: wgpu::BufferUsages::STORAGE
            | wgpu::BufferUsages::COPY_DST
            | wgpu::BufferUsages::COPY_SRC,
    });

    let output_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Compute output buffer"),
        size: (NUM_BOIDS * 4 * std::mem::size_of::<f32>()) as u64,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
        mapped_at_creation: false,
    });

    let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("Compute bind group layout"),
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: wgpu::BufferSize::new(
                        (NUM_BOIDS * 4 * std::mem::size_of::<f32>()) as u64,
                    ),
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: false },
                    has_dynamic_offset: false,
                    min_binding_size: wgpu::BufferSize::new(
                        (NUM_BOIDS * 4 * std::mem::size_of::<f32>()) as u64,
                    ),
                },
                count: None,
            },
        ],
    });

    let compute_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("nannou"),
        bind_group_layouts: &[&bind_group_layout],
        push_constant_ranges: &[],
    });

    let compute_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: Some("Compute pipeline"),
        layout: Some(&compute_pipeline_layout),
        module: &cs_module,
        entry_point: "main",
    });

    let compute = Compute {
        buffer_size,
        boids_buffer,
        output_buffer,
        bind_group_layout,
        compute_pipeline,
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
    // update_boids(&mut model.boids, update.since_last.as_secs_f32());

    let window = _app.main_window();
    let device = window.device();
    let compute = &mut model.compute;

    let staging_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Staging buffer"),
        size: model.compute.buffer_size,
        usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("Compute pass encoder"),
    });
    {
        let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("Compute pass descriptor"),
        });
        pass.set_pipeline(&model.compute.compute_pipeline);
    }
    encoder.copy_buffer_to_buffer(
        &model.compute.output_buffer,
        0,
        &staging_buffer,
        0,
        model.compute.buffer_size,
    );

    let mut data = Vec::with_capacity(NUM_BOIDS);
    for boid in &model.boids {
        data.push([boid.x, boid.y, boid.vx, boid.vy]);
    }
    let boids_slice = &data[..NUM_BOIDS];
    let boids_bytes: &[u8] = unsafe { wgpu::bytes::from_slice(&boids_slice) };

    window
        .queue()
        .write_buffer(&model.compute.boids_buffer, 0, &boids_bytes);

    window.queue().submit(Some(encoder.finish()));

    // Async result read

    let future = async move {
        let staging_slice = staging_buffer.slice(..);
        let (tx, rx) = flume::bounded(1);

        staging_slice.map_async(wgpu::MapMode::Read, move |v| tx.send(v).unwrap());
        device.poll(wgpu::Maintain::Wait);

        if let Ok(Ok(())) = rx.recv_async().await {
            let data = staging_slice.get_mapped_range();
            let result: Vec<Boid> = bytemuck::cast_slice(&data).to_vec();

            drop(data);

            staging_buffer.unmap();
            model.boids = result;
        }
    };
    pollster::block_on(future);
}

fn view(app: &App, model: &Model, frame: Frame) {
    // Encoder encodes the commands that are later dispatched to the GPU queue
    let mut encoder = frame.command_encoder();

    // Build render pass clearing the screen before each frame
    let mut render_pass = wgpu::RenderPassBuilder::new()
        .color_attachment(frame.texture_view(), |color| {
            color
                .load_op(wgpu::LoadOp::Clear(wgpu::Color {
                    r: 0.0,
                    g: 0.0,
                    b: 0.0,
                    a: 0.5,
                }))
                .store_op(true)
        })
        .begin(&mut encoder);

    // Bind resources to the render pass
    render_pass.set_bind_group(0, &model.render.bind_group, &[]);
    render_pass.set_pipeline(&model.render.render_pipeline);
    render_pass.set_vertex_buffer(0, model.render.vertex_buffer.slice(..));

    // Convert the vector of Boids to a &[u8]
    // TODO : manage clip space coordinates
    let mut vertices = Vec::with_capacity(NUM_BOIDS);
    for boid in &model.boids {
        vertices.push(Vertex {
            position: [boid.x / 500.0, boid.y / 500.0],
        });
    }
    let vertices_slice: &[Vertex] = &vertices[..NUM_BOIDS];
    let vertex_bytes: &[u8] = unsafe { wgpu::bytes::from_slice(&vertices_slice) };

    // Submit command to the queue
    app.main_window()
        .queue()
        .write_buffer(&model.render.vertex_buffer, 0, &vertex_bytes);

    // Performs the render pass
    render_pass.draw(0..NUM_BOIDS as u32, 0..1);
}
