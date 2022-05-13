


use pollster;
use winit::{
	event::*,
	event_loop::{ ControlFlow, EventLoop },
	window::{ WindowBuilder, Window }
};
use wgpu;
use wgpu::util::DeviceExt;

use std::time::{ Instant, Duration };



#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
	position: [f32; 2]
}

impl Vertex {
	fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
		wgpu::VertexBufferLayout {
			array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
			step_mode: wgpu::VertexStepMode::Vertex,
			attributes: &[
				wgpu::VertexAttribute {
					offset: 0,
					shader_location: 0,
					format: wgpu::VertexFormat::Float32x2
				}
			]
		}
	}
}

const VERTICES: [Vertex; 4] = [
	Vertex { position: [-1.0, -1.0] },
	Vertex { position: [-1.0,  1.0] },
	Vertex { position: [ 1.0,  1.0] },
	Vertex { position: [ 1.0, -1.0] },
];

const INDICES: &[u16] = &[
	2, 1, 0,
	3, 2, 0
];



#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct Uniforms {
	time: f32
}

impl Uniforms {
	fn new() -> Self {
		Self {
			time: 0.0
		}
	}

	fn update(&mut self, time: &Duration) {
		self.time = time.as_secs_f32();
	}
}



async fn run(event_loop: EventLoop<()>, window: Window) {
	
	let mut state = pollster::block_on(State::new(&window));

	let mut last_render_time = Instant::now();

	event_loop.run(move |event, _, control_flow| match event {
		// Event::DeviceEvent {
		// 	event: DeviceEvent::MouseMotion { delta },
		// 	..
		// } => 
		Event::WindowEvent {
			ref event,
			window_id
		} if window_id == window.id() => {
			if !state.input(event) {
				match event {
					WindowEvent::CloseRequested
					| WindowEvent::KeyboardInput {
						input: KeyboardInput {
							state: ElementState::Pressed,
							virtual_keycode: Some(VirtualKeyCode::Escape),
							..
						},
						..
					} => *control_flow = ControlFlow::Exit,
					_ => {}
				}
			}
		},
		Event::RedrawRequested(window_id) if window_id == window.id() => {
			let now = Instant::now();
			let dt = now - last_render_time;
			last_render_time = now;
			state.update(dt);
			match state.render() {
				Ok(_) => {},
				Err(wgpu::SurfaceError::Lost) => state.resize(state.size),
				Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
				Err(e) => eprintln!("{:?}", e)
			}
		},
		Event::RedrawEventsCleared => {
			window.request_redraw();
		},
		_ => {}
	});

}



struct State {
	surface: wgpu::Surface,
	device: wgpu::Device,
	queue: wgpu::Queue,
	config: wgpu::SurfaceConfiguration,
	size: winit::dpi::PhysicalSize<u32>,
	clear_color: wgpu::Color,
	render_pipeline: wgpu::RenderPipeline,
	vertices: [Vertex; 4],
	vertex_buffer: wgpu::Buffer,
	num_vertices: u32,
	index_buffer: wgpu::Buffer,
	num_indices: u32,
	time: Duration,
	uniforms: Uniforms,
	uniforms_buffer: wgpu::Buffer,
	uniforms_bind_group: wgpu::BindGroup
}

impl State {

	async fn new(window: &Window) -> Self {

		let size = window.inner_size();

		let instance = wgpu::Instance::new(wgpu::Backends::all());
		let surface = unsafe { instance.create_surface(&window) };
		let adapter = instance.request_adapter(
			&wgpu::RequestAdapterOptions {
				power_preference: wgpu::PowerPreference::HighPerformance,
				compatible_surface: Some(&surface),
				// force_fallback_adapter: true
				force_fallback_adapter: false
			}
		).await.expect("Couldn't make adapter");

		let (device, queue) = adapter.request_device(
			&wgpu::DeviceDescriptor {
				features: wgpu::Features::empty(),
				limits: wgpu::Limits::downlevel_webgl2_defaults().using_resolution(adapter.limits()),
				label: Some("Device Descriptor")
			},
			None
		).await.unwrap();

		let config = wgpu::SurfaceConfiguration {
			usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
			format: surface.get_preferred_format(&adapter).expect("Couldn't get required adapter preferences"),
			width: size.width,
			height: size.height,
			present_mode: wgpu::PresentMode::Fifo
		};
		surface.configure(&device, &config);


		let clear_color = wgpu::Color::BLUE;

		let shader = device.create_shader_module(&wgpu::ShaderModuleDescriptor {
			label: Some("shader"),
			source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into())
		});


		let vertices = VERTICES;

		let vertex_buffer = device.create_buffer_init(
			&wgpu::util::BufferInitDescriptor {
				label: Some("Vertex Buffer"),
				contents: bytemuck::cast_slice(&vertices),
				usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST
			}
		);
		let num_vertices = vertices.len() as u32;

		let index_buffer = device.create_buffer_init(
			&wgpu::util::BufferInitDescriptor {
				label: Some("Index Buffer"),
				contents: bytemuck::cast_slice(INDICES),
				usage: wgpu::BufferUsages::INDEX
			}
		);
		let num_indices = INDICES.len() as u32;


		let time = Duration::from_secs(0);

		let mut uniforms = Uniforms::new();
		uniforms.update(&time);

		let uniforms_buffer = device.create_buffer_init(
			&wgpu::util::BufferInitDescriptor {
				label: Some("Uniforms Buffer"),
				contents: bytemuck::cast_slice(&[uniforms]),
				usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST
			}
		);

		let uniforms_bind_group_layout = device.create_bind_group_layout(
			&wgpu::BindGroupLayoutDescriptor {
				entries: &[
					wgpu::BindGroupLayoutEntry {
						binding: 0,
						visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
						ty: wgpu::BindingType::Buffer {
							ty: wgpu::BufferBindingType::Uniform,
							has_dynamic_offset: false,
							min_binding_size: None
						},
						count: None
					}
				],
				label: Some("Uniforms Bind Group Layout")
			}
		);

		let uniforms_bind_group = device.create_bind_group(
			&wgpu::BindGroupDescriptor {
				layout: &uniforms_bind_group_layout,
				entries: &[
					wgpu::BindGroupEntry {
						binding: 0,
						resource: uniforms_buffer.as_entire_binding()
					}
				],
				label: Some("Uniforms Bind Group")
			}
		);


		let render_pipeline_layout = device.create_pipeline_layout(
			&wgpu::PipelineLayoutDescriptor {
				label: Some("Render Pipeline Layout"),
				bind_group_layouts: &[
					&uniforms_bind_group_layout
				],
				push_constant_ranges: &[]
			}
		);

		let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
			label: Some("Render Pipeline"),
			layout: Some(&render_pipeline_layout),
			vertex: wgpu::VertexState {
				module: &shader,
				entry_point: "vertex_shader_main",
				buffers: &[
					Vertex::desc()
				]
			},
			fragment: Some(wgpu::FragmentState {
				module: &shader,
				entry_point: "fragment_shader_main",
				targets: &[wgpu::ColorTargetState {
					format: config.format,
					blend: Some(wgpu::BlendState::REPLACE),
					write_mask: wgpu::ColorWrites::ALL
				}]
			}),
			primitive: wgpu::PrimitiveState {
				topology: wgpu::PrimitiveTopology::TriangleList,
				strip_index_format: None,
				front_face: wgpu::FrontFace::Ccw,
				cull_mode: Some(wgpu::Face::Back),
				polygon_mode: wgpu::PolygonMode::Fill,
				unclipped_depth: false,
				conservative: false
			},
			depth_stencil: None,
			multisample: wgpu::MultisampleState {
				count: 1,
				mask: 10,
				alpha_to_coverage_enabled: false
			},
			multiview: None
		});


		Self {
			surface, device, queue, config, size, clear_color, render_pipeline,
			vertices, vertex_buffer, num_vertices, index_buffer, num_indices,
			time, uniforms, uniforms_buffer, uniforms_bind_group
		}
	}



	fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
		if new_size.width > 0 && new_size.height > 0 {
			self.size = new_size;
			self.config.width = new_size.width;
			self.config.height = new_size.height;
			self.surface.configure(&self.device, &self.config);
		}
	}



	fn input(&mut self, event: &WindowEvent) -> bool {
		false
	}



	fn update(&mut self, dt: std::time::Duration) {
		self.time += dt;
		self.uniforms.update(&self.time);
		self.queue.write_buffer(&self.uniforms_buffer, 0, bytemuck::cast_slice(&[self.uniforms]));
	}



	fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
		let output = self.surface.get_current_texture()?;
		let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
		let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
			label: Some("Render Encoder")
		});

		{
			let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
				label: Some("Render Pass"),
				color_attachments: &[
					wgpu::RenderPassColorAttachment {
						view: &view,
						resolve_target: None,
						ops: wgpu::Operations {
							load: wgpu::LoadOp::Clear(self.clear_color),
							store: true
						}
					}
				],
				depth_stencil_attachment: None
			});

			render_pass.set_pipeline(&self.render_pipeline);
			render_pass.set_bind_group(0, &self.uniforms_bind_group, &[]);
			render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
			render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
			render_pass.draw_indexed(0..self.num_indices, 0, 0..1);
		}

		self.queue.submit(std::iter::once(encoder.finish()));
		output.present();

		Ok(())
	}

}



fn main() {

	let event_loop = EventLoop::new();
	let window = WindowBuilder::new().build(&event_loop).unwrap();

	pollster::block_on(run(event_loop, window));

}


