mod utilities;
mod raytracer;
mod geometry;

use wgpu::util::DeviceExt;
use winit::{
    event::*,
    dpi::PhysicalSize,
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};
use winit::window::{Window, WindowId};
use winit::event::WindowEvent;

use geometry::{ Vertex };
use raytracer::{ Raytracer, init_raytracer };

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
    env_logger::init();
    let event_loop = EventLoop::new();
    let canvas_size = Raytracer::CANVAS;
    let window_size = PhysicalSize::new(canvas_size[0], canvas_size[1]);
    let window = WindowBuilder::new()
        .with_title("Raytracer")
        .with_inner_size(window_size)
        .build(&event_loop)
        .unwrap();

    let mut raytracer = init_raytracer();
    let vertices = raytracer.get_state();
    let mut state = State::new(window, vertices).await;

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
