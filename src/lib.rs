use std::ffi::c_short;
use std::ops::RangeInclusive;
use std::os::unix::raw::dev_t;
use wgpu::util::DeviceExt;
use winit::{
    event::*,
    dpi::PhysicalSize,
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};
use winit::window::{Window, WindowId};
use winit::event::WindowEvent;

fn dot_product(v1: [f32; 3], v2: [f32; 3]) -> f32 {
    v1[0] * v2[0] + v1[1] * v2[1] + v1[2] * v2[2]
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 3],
    color: [f32; 3],
}
impl Vertex {
    const ATTRIBS: [wgpu::VertexAttribute; 2] = [
        wgpu::VertexAttribute {
            offset: 0,
            shader_location: 0,
            format: wgpu::VertexFormat::Float32x3
        },
        wgpu::VertexAttribute {
            offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
            shader_location: 1,
            format: wgpu::VertexFormat::Float32x3
        }
    ];
    fn describe() -> wgpu::VertexBufferLayout<'static> {
        return wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        };
    }
}

struct Sphere {
    radius: f32,
    center: [f32; 3],
    color: [f32; 3],
}

impl Sphere {
    pub fn intersect_ray(&self, camera_origin: [f32; 3], ray_direction: [f32; 3]) -> (f32, f32) {
        let distance_to_center = [
            camera_origin[0] - self.center[0],
            camera_origin[1] - self.center[1],
            camera_origin[2] - self.center[2],
        ];

        let a = dot_product(ray_direction, ray_direction);
        let b = 2.0 * dot_product(distance_to_center, ray_direction);
        let c = dot_product(distance_to_center, distance_to_center) - self.radius * self.radius;
        let discriminant = b * b - 4.0 * a * c;

        if discriminant < 0.0 {
            return (f32::INFINITY, f32::INFINITY);
        }

        let t1 = (-b + discriminant.sqrt()) / (2.0 * a);
        let t2 = (-b - discriminant.sqrt()) / (2.0 * a);

        return (t1, t2);
    }
}

struct Raytracer {
    state: Vec<Vertex>,
    scene: Vec<Sphere>,
}
impl Raytracer {
    const CAMERA_POSITION: [f32; 3] = [0.0, 0.0, 0.0];
    // Camera 3D position

    const VIEWPORT: [f32; 3] = [1.0, 1.0, 1.0];
    // Viewport width, height and depth which is camera distance

    const CANVAS: [i32; 2] = [ 1600, 1600 ];
    // Canvas size

    const BACKGROUND_COLOR: [f32; 3] = [1.0, 1.0, 1.0];
    // Default color for scene

    pub fn new() -> Self {
        Self { state: vec![], scene: vec![] }
    }
    pub fn put_pixel(&mut self, x: i32, y: i32, color: [f32; 3]) {
        let x_cord = x as f32 / (Self::CANVAS[0] / 2) as f32;
        let y_cord = y as f32 / (Self::CANVAS[1] / 2) as f32;
        self.state.push(Vertex { position: [x_cord, y_cord, 0.0], color });
    }
    pub fn get_state(&mut self) -> &[Vertex] {
        return self.state.as_slice();
    }
    pub fn get_canvas_size(&mut self) -> [i32; 2] {
        return Self::CANVAS;
    }

    pub fn add_to_scene(&mut self, sphere: Sphere) {
        self.scene.push(sphere);
    }

    fn get_canvas_range(&mut self, axis: char) -> RangeInclusive<i32> {
        match axis {
            'x' => { ( -Self::CANVAS[0]/2 ..= Self::CANVAS[0]/2 ) },
            'y' => { ( -Self::CANVAS[1]/2 ..= Self::CANVAS[1]/2 ) },
            _ => { ( -1 ..= 1 ) }
        }
    }

    fn canvas_to_viewport(&mut self, x: i32, y: i32) -> [f32; 3] {
        let x_pos = x as f32 * Self::VIEWPORT[0] / Self::CANVAS[0] as f32;
        let y_pos = y as f32 * Self::VIEWPORT[1] / Self::CANVAS[1] as f32;
        let z_pos = Self::VIEWPORT[2];
        return [x_pos, y_pos, z_pos];
    }

    fn trace_ray(&mut self, direction: [f32; 3], t_min: f32, t_max: f32) -> [f32; 3] {
        let mut closest_t = f32::INFINITY;
        let mut closest_sphere: Option<&Sphere> = None;
        let ray_range = (t_min ..= t_max);

        for sphere in self.scene.iter() {
            let (t1, t2) = sphere.intersect_ray(Self::CAMERA_POSITION, direction);

            if ray_range.contains(&t1) && t1 < closest_t {
                closest_t = t1;
                closest_sphere = Some(sphere);
            }

            if ray_range.contains(&t2) && t2 < closest_t {
                closest_t = t1;
                closest_sphere = Some(sphere);
            }
        }

        match closest_sphere {
            Some(sphere) => sphere.color,
            None => Self::BACKGROUND_COLOR,
        }
    }

    pub fn pass(&mut self) {
        for x in self.get_canvas_range('x').clone() {
            for y in self.get_canvas_range('y').clone() {
                let direction = self.canvas_to_viewport(x, y);
                let color = self.trace_ray(direction, 1.0, f32::INFINITY);
                self.put_pixel(x, y, color);
            }
        }

    }
}

pub struct State {
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    render_pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    num_vertices: u32,
    size: winit::dpi::PhysicalSize<u32>,
    window: Window,
}

impl State {
    pub fn window(&self) -> &Window {
        &self.window
    }
    async fn new(window: Window, vertices: &[Vertex]) -> Self {
        let size = window.inner_size();
        let instance_options = wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        };
        let instance = wgpu::Instance::new(instance_options);
        let surface = unsafe { instance
            .create_surface(&window)
            .unwrap()
        };
        let adapter_options = &wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: Some(&surface),
            force_fallback_adapter: false
        };
        let adapter = instance
            .request_adapter(adapter_options)
            .await.unwrap();
        let device_options = &wgpu::DeviceDescriptor {
            features: wgpu::Features::empty(),
            limits: wgpu::Limits::default(),
            label: None,
        };
        let (device, queue) = adapter
            .request_device(device_options, None)
            .await.unwrap();
        let surface_capabilities = surface.get_capabilities(&adapter);
        let surface_format = surface_capabilities.formats
            .iter()
            .copied()
            .filter(|f| f.is_srgb())
            .next()
            .unwrap_or(surface_capabilities.formats[0]);
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_capabilities.present_modes[0],
            alpha_mode: surface_capabilities.alpha_modes[0],
            view_formats: vec![],
        };
        surface.configure(&device, &config);
        let shader_module_descriptor = wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        };
        let shader = device.create_shader_module(shader_module_descriptor);
        let render_pipeline_layout_descriptor = &wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[],
            push_constant_ranges: &[],
        };
        let render_pipeline_layout = device.create_pipeline_layout(render_pipeline_layout_descriptor);
        let vertex_state = wgpu::VertexState {
            module: &shader,
            entry_point: "vs_main",
            buffers: &[
                Vertex::describe(),
            ]
        };
        let color_target_state = wgpu::ColorTargetState {
            format: config.format,
            blend: Some(wgpu::BlendState::REPLACE),
            write_mask: wgpu::ColorWrites::ALL
        };
        let fragment_state = wgpu::FragmentState {
            module: &shader,
            entry_point: "fs_main",
            targets: &[Some(color_target_state)]
        };
        let primitive_state = wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::PointList,
            strip_index_format: None,
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: Some(wgpu::Face::Back),
            polygon_mode: wgpu::PolygonMode::Fill,
            unclipped_depth: false,
            conservative: false,
        };
        let multisample_state = wgpu::MultisampleState {
            count: 1,
            mask: !0,
            alpha_to_coverage_enabled: false,
        };
        let render_pipeline_descriptor = &wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: vertex_state,
            fragment: Some(fragment_state),
            primitive: primitive_state,
            multisample: multisample_state,
            depth_stencil: None,
            multiview: None,
        };
        let render_pipeline = device.create_render_pipeline(render_pipeline_descriptor);
        let vertex_buffer_descriptor = &wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(vertices),
            usage: wgpu::BufferUsages::VERTEX,
        };
        let vertex_buffer = device.create_buffer_init(vertex_buffer_descriptor);
        let num_vertices = vertices.len() as u32;

        Self {
            surface,
            device,
            queue,
            config,
            render_pipeline,
            vertex_buffer,
            num_vertices,
            size,
            window,
        }
    }

    fn input(&mut self, _event: &WindowEvent) -> bool {
        false
    }

    fn update(&mut self) {}

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

        let encoder_options = &wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        };
        let mut encoder = self.device.create_command_encoder(encoder_options);

        {
            let bg_color = wgpu::Color {
                r: 0.1,
                g: 0.2,
                b: 0.3,
                a: 1.0,
            };
            let pass_color_attachment = wgpu::RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(bg_color),
                    store: wgpu::StoreOp::Store,
                },
            };
            let pass_options = &wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(pass_color_attachment)],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            };

            let mut render_pass = encoder.begin_render_pass(pass_options);
            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.draw(0..self.num_vertices, 0..1);
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.height > 0 && new_size.width > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
        }
    }
}

pub async fn run() {
    let mut raytracer = Raytracer::new();

    env_logger::init();
    let event_loop = EventLoop::new();
    let canvas_size = raytracer.get_canvas_size();
    let window_size = PhysicalSize::new(canvas_size[0], canvas_size[1]);
    let window = WindowBuilder::new()
        .with_title("Raytracer")
        .with_inner_size(window_size)
        .build(&event_loop)
        .unwrap();

    raytracer.add_to_scene(Sphere {
        radius: 1.0,
        center: [0.0, -1.0, 3.0],
        color: [1.0, 0.0, 0.0]
    });

    raytracer.add_to_scene(Sphere {
        radius: 1.0,
        center: [0.0, -1.0, 3.0],
        color: [1.0, 0.0, 0.0]
    });

    raytracer.add_to_scene(Sphere {
        radius: 1.0,
        center: [2.0, 0.0, 4.0],
        color: [0.0, 0.0, 1.0]
    });

    raytracer.add_to_scene(Sphere {
        radius: 1.0,
        center: [-2.0, 0.0, 4.0],
        color: [0.0, 1.0, 0.0]
    });

    raytracer.pass();

    let mut state = State::new(window, raytracer.get_state()).await;

    event_loop.run(move |event, _, control_flow|
        match event {
        Event::RedrawRequested(window_id) => {
            handle_redraw(&mut state, window_id, control_flow);
        }
        Event::MainEventsCleared => {
            state.window().request_redraw();
        }
        Event::WindowEvent { ref event, window_id } => {
            handle_window(&mut state, window_id, control_flow, event)
        }
        _ => {}
    });
}

fn handle_redraw(state: &mut State, window_id: WindowId, control_flow: &mut ControlFlow) {
    if window_id == state.window().id() {
        state.update();
        match state.render() {
            Ok(_) => {}
            Err(wgpu::SurfaceError::Lost) => state.resize(state.size),
            Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
            Err(e) => eprintln!("{:?}", e),
        }
    }
}

fn handle_window(state: &mut State, window_id: WindowId, control_flow: &mut ControlFlow, event: &WindowEvent) {
    if window_id == state.window().id() && !state.input(event) {
        match event {
            WindowEvent::CloseRequested | WindowEvent::KeyboardInput {
                input:
                KeyboardInput {
                    state: ElementState::Pressed,
                    virtual_keycode: Some(VirtualKeyCode::Escape),
                    ..
                },
                ..
            } => {
                *control_flow = ControlFlow::Exit
            },
            WindowEvent::Resized(physical_size) => {
                state.resize(*physical_size);
            }
            WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                state.resize(**new_inner_size);
            }
            _ => {}
        }
    }
}
