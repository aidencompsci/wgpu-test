#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

use winit::{
	event::*,
	event_loop::{ControlFlow, EventLoop},
	window::WindowBuilder,
};

use winit::window::Window;

struct State {
	surface: wgpu::Surface,
	device: wgpu::Device,
	queue: wgpu::Queue,
	config: wgpu::SurfaceConfiguration,
	size: winit::dpi::PhysicalSize<u32>,
	clear_color: wgpu::Color,
	render_pipeline: wgpu::RenderPipeline,
}

impl State {
	async fn new(window: &Window) -> Self {
		let size = window.inner_size();

		// instance is a gpu handler
		// backends:;all => vulkan + metal + dx12 + browser web gpu
		let instance = wgpu::Instance::new(wgpu::Backends::all());
		let surface = unsafe {instance.create_surface(window) };

		let adapter = instance.request_adapter(
			&wgpu::RequestAdapterOptions {
				power_preference: wgpu::PowerPreference::HighPerformance,
				compatible_surface: Some(&surface),
				force_fallback_adapter: false,
			},
		).await.unwrap();

		let (device, queue) = adapter.request_device(
			&wgpu::DeviceDescriptor {
				features: wgpu::Features::empty(),
				limits: if cfg!(target_arch = "wasm32") {
					wgpu::Limits::downlevel_webgl2_defaults()
				}
				else {
					wgpu::Limits::default()
				},
				label: None,
			},
			None,
		).await.unwrap();

		let config = wgpu::SurfaceConfiguration {
			usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
			format: surface.get_supported_formats(&adapter)[0],
			width: size.width,
			height: size.height,
			present_mode: wgpu::PresentMode::Fifo,
			alpha_mode: wgpu::CompositeAlphaMode::Auto,
		};
		surface.configure(&device, &config);

		let clear_color = wgpu::Color::BLACK;

        // let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        //     label: Some("Shader"),
        //     source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        // });

        let shader = device.create_shader_module(wgpu::include_wgsl!("shader.wgsl"));

        let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("render Pipeline Layout"),
            bind_group_layouts: &[],
            push_constant_ranges: &[],
        });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        });

		Self {
			surface,
			device,
			queue,
			config,
			size,
			clear_color,
            render_pipeline,
		}

	}

	fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
		if new_size.width > 0 && new_size.height > 0 {
			self.size = new_size;
			self.config.width = new_size.width;
			self.config.height = new_size.height;
			self.surface.configure(&self.device, &self.config)
		}
	}

	fn input(&mut self, event: &WindowEvent) -> bool {
		match event {
			WindowEvent::CursorMoved { position, .. } => {
				self.clear_color = wgpu::Color {
					r: position.x as f64 / self.size.width as f64,
					g: position.y as f64 / self.size.height as f64,
					b: 1.0,
					a: 1.0
				};
				return true;
			},
			_ => return false,
		}
	}

	fn update(&mut self) {

	}

	fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
		let output = self.surface.get_current_texture()?;
		let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
		let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
			label: Some("Render Encoder"),
		});
		{
			let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
				label: Some("Render Pass"),
				color_attachments: &[Some(wgpu::RenderPassColorAttachment {
					view: &view,
					resolve_target: None,
					ops: wgpu::Operations {
						load: wgpu::LoadOp::Clear(self.clear_color),
						store: true,
					},
				})],
				depth_stencil_attachment: None,
			});
            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.draw(0..3, 0..1);
		}

		self.queue.submit(std::iter::once(encoder.finish()));
		output.present();

		Ok(())
	}
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(start))]
pub async fn run() {
	cfg_if::cfg_if! {
		if #[cfg(target_arch = "wasm32")] {
			std::panic::set_hook(Box::new(console_error_panic_hook::hook));
			console_log::init_with_level(log::Level::Warn).expect("Couldn't initialize logger");
		}
		else {
			env_logger::init();
		}
	}

	let event_loop = EventLoop::new();
	let window = WindowBuilder::new().build(&event_loop).unwrap();

	#[cfg(target_arch = "wasm32")] {
		use winit::dpi::PhysicalSize;
		window.set_inner_size(PhysicalSize::new(450, 400));

		use winit::platform::web::WindowExtWebSys;;
		web_sys::window()
			.and_then(|win| win.document())
			.and_then(|doc| {
				let dst = doc.get_element_by_id("wgpu-test")?;
				let canvas = web_sys::Element::from(window.canvas());
				dst.append_child(&canvas).ok()?;
				Some(())
			})
			.expect("Couldn't append canvas to document body")
	}

	let mut state = State::new(&window).await;

	event_loop.run(move |event, _, control_flow| match event {
		Event::WindowEvent {ref event, window_id}
		if window_id == window.id() =>
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
				} =>
					*control_flow = ControlFlow::Exit,
				WindowEvent::Resized(physical_size) => {
					state.resize(*physical_size);
				}
				WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
					state.resize(**new_inner_size);
				}
				_ => {}
			}
		},
		Event::RedrawRequested(window_id)
		if window_id == window.id() => {
			state.update();
			match state.render() {
				Ok(_) => {},
				Err(wgpu::SurfaceError::Lost) => state.resize(state.size),
				Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
				Err(e) => eprintln!("{:?}", e),
			}
		},
		Event::MainEventsCleared => {
			window.request_redraw();
		}
		_ => {}
	});
}
