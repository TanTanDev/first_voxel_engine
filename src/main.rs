use camera::Camera;
use camera_controller::*;
use cgmath::{InnerSpace, Zero};
use futures::executor::block_on;
use model::Model;
use rendering::gpu_resources::GpuResources;
use voxel_tools::chunks::Chunks;
use wgpu::util::DeviceExt;
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

use crate::{
    depth_pass::DepthPass,
    light::Light,
    rendering::{
        render_utils::create_render_pipeline, vertex_desc::VertexDesc, vertex_instance::*,
    },
    voxel_tools::{
        rendering::voxel_pipeline::create_voxel_pipeline,
    },
};

mod camera;
mod camera_controller;
mod color;
mod depth_pass;
mod light;
mod model;
mod rendering;
mod texture;
mod voxel_tools;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub position: [f32; 3],
    //color: [f32; 3],
    pub tex_coords: [f32; 2],
}

pub const NUM_INSTANCES_PER_ROW: u32 = 100;
pub const NUM_INSTANCES: u32 = NUM_INSTANCES_PER_ROW * NUM_INSTANCES_PER_ROW;
pub const INSTANCE_DISPLACEMENT: cgmath::Vector3<f32> = cgmath::Vector3::new(
    NUM_INSTANCES_PER_ROW as f32 * 0.5f32,
    0.0,
    NUM_INSTANCES_PER_ROW as f32 * 0.5f32,
);

struct State {
    gpu_resources: GpuResources,
    instance_buffer: wgpu::Buffer,
    rotation: f32,
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    sc_desc: wgpu::SwapChainDescriptor,
    swap_chain: wgpu::SwapChain,
    render_pipeline: wgpu::RenderPipeline,
    size: winit::dpi::PhysicalSize<u32>,
    clear_color: wgpu::Color,
    camera_uniform_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,
    camera: Camera,
    camera_controller: CameraController,
    depth_pass: depth_pass::DepthPass,
    obj_model: Model,

    light_render_pipeline: wgpu::RenderPipeline,
    light_bind_group: wgpu::BindGroup,
    light: Light,
    light_buffer: wgpu::Buffer,

    voxel_render_pipeline: wgpu::RenderPipeline,
    mouse_pressed: bool,

    chunks: Chunks,
}

impl State {
    async fn new(window: &Window) -> Self {
        let size = window.inner_size();

        // handler for our gpu
        let instance = wgpu::Instance::new(wgpu::BackendBit::DX12);
        let surface = unsafe { instance.create_surface(window) };
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
            })
            .await
            .unwrap();
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    //features: wgpu::Features::empty(),
                    features: wgpu::Features::NON_FILL_POLYGON_MODE,
                    limits: wgpu::Limits::default(),
                    label: None,
                },
                None, // trace path
            )
            .await
            .unwrap();
        let sc_desc = wgpu::SwapChainDescriptor {
            usage: wgpu::TextureUsage::RENDER_ATTACHMENT,
            format: adapter
                .get_swap_chain_preferred_format(&surface)
                .unwrap_or(wgpu::TextureFormat::Rgba8UnormSrgb),
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
        };
        let swap_chain = device.create_swap_chain(&surface, &sc_desc);

        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStage::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: false },
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStage::FRAGMENT,
                        ty: wgpu::BindingType::Sampler {
                            comparison: false,
                            filtering: true,
                        },
                        count: None,
                    },
                ],
                label: Some("texture_bind_group_layout"),
            });

        let light = Light {
            position: [50.0, 2.0, 50.0],
            _padding: 0,
            color: [1.0, 1.0, 1.0],
        };

        let light_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("light buffer"),
            contents: bytemuck::cast_slice(&[light]),
            usage: wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
        });

        let light_bind_group_layout = light::create_light_bind_group_layout(&device);

        let light_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("light bind group"),
            layout: &light_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: light_buffer.as_entire_binding(),
            }],
        });

        let clear_color = wgpu::Color {
            r: 0.1,
            g: 0.2,
            b: 0.3,
            a: 1.0,
        };
        let aspect = sc_desc.width as f32 / sc_desc.height as f32;
        let mut camera = Camera::new(aspect);

        let offset = 8f32;
        let instances = (0..NUM_INSTANCES_PER_ROW)
            .flat_map(|z| {
                (0..NUM_INSTANCES_PER_ROW).map(move |x| {
                    let position = cgmath::Vector3::new(x as f32 * offset, 0f32, z as f32 * offset)
                        - INSTANCE_DISPLACEMENT;
                    let rotation = if position.is_zero() {
                        cgmath::Quaternion::from_axis_angle(
                            cgmath::Vector3::unit_z(),
                            cgmath::Deg(0.0),
                        )
                    } else {
                        cgmath::Quaternion::from_axis_angle(
                            position.clone().normalize(),
                            cgmath::Deg(45.0),
                        )
                    };
                    //let rotation = cgmath::Quaternion::from_axis_angle(cgmath::Vector3::unit_z(), cgmath::Deg(0.0));
                    VertexInstance { position, rotation }
                })
            })
            .collect::<Vec<_>>();
        use cgmath::Rotation3;

        let instance_data = instances
            .iter()
            .map(VertexInstance::to_raw)
            .collect::<Vec<_>>();
        let instance_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("instance_buffer"),
            contents: bytemuck::cast_slice(&instance_data),
            usage: wgpu::BufferUsage::VERTEX,
        });

        camera.update_uniform();
        let camera_uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("uniform buffer"),
            contents: bytemuck::cast_slice(&[camera.uniform]),
            usage: wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
        });

        let camera_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("uniform_bind_group_layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStage::VERTEX | wgpu::ShaderStage::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });

        let uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_uniform_buffer.as_entire_binding(),
            }],
            label: Some("uniform_bind_group"),
        });

        let shader_flags = wgpu::ShaderFlags::empty();
        use std::borrow::Cow;
        let shader_module = device.create_shader_module(&wgpu::ShaderModuleDescriptor {
            label: Some("shader"),
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("shader.wgsl"))),
            flags: shader_flags,
        });

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[
                    &camera_bind_group_layout,
                    &texture_bind_group_layout,
                    &light_bind_group_layout,
                ],
                push_constant_ranges: &[],
            });

        println!("creating pipeline");
        let render_pipeline = create_render_pipeline(
            &device,
            &render_pipeline_layout,
            sc_desc.format,
            Some(texture::Texture::DEPTH_FORMAT),
            &[model::ModelVertex::desc(), VertexInstanceRaw::desc()],
            shader_module,
            "render_pipeline",
        );

        let shader_module = device.create_shader_module(&wgpu::ShaderModuleDescriptor {
            label: Some("light-shader"),
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("light.wgsl"))),
            flags: shader_flags,
        });

        let light_render_pipeline = {
            let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("light_pipeline_layout"),
                bind_group_layouts: &[&camera_bind_group_layout, &light_bind_group_layout],
                push_constant_ranges: &[],
            });
            create_render_pipeline(
                &device,
                &layout,
                sc_desc.format,
                Some(texture::Texture::DEPTH_FORMAT),
                &[model::ModelVertex::desc()],
                shader_module,
                "light_render_pipeline",
            )
        };

        let camera_controller = CameraController::new(10.2, 1.0);
        let depth_pass = DepthPass::new(&device, &sc_desc);

        let obj_model = model::Model::load(
            &device,
            &queue,
            &texture_bind_group_layout,
            std::path::Path::new("res/turkey.obj"),
        )
        .unwrap();

        let voxel_render_pipeline =
            create_voxel_pipeline(&device, sc_desc.format, &light_bind_group_layout);

        let mut gpu_resources = GpuResources::new();

        let mut chunks = Chunks::new();
        // find what chunks needs to be loaded
        chunks.update_load_data_queue();
        chunks.update_load_mesh_queue();

        // load voxel data in chunks
        chunks.build_chunk_data_in_queue();

        // load meshes based on voxel data in chunk
        chunks.build_chunk_meshes_in_queue(&device, &mut gpu_resources);

        Self {
            gpu_resources,
            chunks,
            rotation: 0f32,
            surface,
            camera,
            camera_controller,
            camera_uniform_buffer,
            camera_bind_group: uniform_bind_group,
            device,
            queue,
            depth_pass,
            sc_desc,
            swap_chain,
            size,
            clear_color,
            render_pipeline,
            instance_buffer,
            obj_model,
            light_bind_group,
            light,
            light_buffer,
            light_render_pipeline,
            voxel_render_pipeline,
            mouse_pressed: false,
        }
    }

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        self.size = new_size;
        self.sc_desc.width = new_size.width;
        self.sc_desc.height = new_size.height;
        self.camera.aspect = new_size.width as f32 / new_size.height as f32;
        self.swap_chain = self.device.create_swap_chain(&self.surface, &self.sc_desc);
        self.depth_pass.resize(&self.device, &self.sc_desc);
    }

    fn input(&mut self, event: &DeviceEvent) -> bool {
        match event {
            DeviceEvent::MouseMotion { delta } => {
                if self.mouse_pressed {
                    self.camera_controller.process_mouse(delta.0, delta.1);
                }
                true
            }
            DeviceEvent::MouseWheel { delta } => {
                self.camera_controller.process_scroll(delta);
                true
            }
            DeviceEvent::Button { button: 1, state } => {
                self.mouse_pressed = *state == ElementState::Pressed;
                true
            }
            DeviceEvent::Key(KeyboardInput {
                virtual_keycode: Some(key),
                state,
                ..
            }) => self.camera_controller.process_keyboard(*key, *state),
            _ => false,
        }
    }

    fn update(&mut self, dt: std::time::Duration) {
        use cgmath::Rotation3;
        let old_position: cgmath::Vector3<_> = self.light.position.into();
        self.light.position =
            (cgmath::Quaternion::from_axis_angle((0.0, 1.0, 0.0).into(), cgmath::Deg(1.0))
                * old_position)
                .into();
        self.queue
            .write_buffer(&self.light_buffer, 0, bytemuck::cast_slice(&[self.light]));

        self.camera_controller.update_camera(&mut self.camera, dt);
        self.camera.update_uniform();
        self.rotation += 3f32;
        self.queue.write_buffer(
            &self.camera_uniform_buffer,
            0,
            bytemuck::cast_slice(&[self.camera.uniform]),
        );

        self.chunks.position = (
            self.camera.position.x,
            self.camera.position.y,
            self.camera.position.z,
        )
            .into();

        use rand::*;
        if rand::thread_rng().gen_range(0..5) == 0 {
            self.chunks.update_load_data_queue();
            self.chunks.update_load_mesh_queue();

            self.chunks.update_unload_mesh_queue();
            self.chunks.update_unload_data_queue();
        }
        self.chunks
            .build_chunk_data_in_queue();
        self.chunks
            .build_chunk_meshes_in_queue(&self.device, &mut self.gpu_resources);
        self.chunks.unload_data_queue();
        self.chunks.unload_mesh_queue(&mut self.gpu_resources);
    }

    fn render(&mut self) -> Result<(), wgpu::SwapChainError> {
        let frame = self.swap_chain.get_current_frame()?.output;
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render encoder"),
            });

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("main render pass"),
            color_attachments: &[wgpu::RenderPassColorAttachment {
                //attachment: &frame.view,
                view: &frame.view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(self.clear_color),
                    store: true,
                },
            }],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: &self.depth_pass.texture.view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: true,
                }),
                stencil_ops: None,
            }),
        });

        render_pass.set_pipeline(&self.voxel_render_pipeline);

        let _ = self.chunks.draw(
            &mut render_pass,
            &self.camera_bind_group,
            &self.light_bind_group,
            &self.gpu_resources,
        );

        let pipeline = &self.render_pipeline;
        use crate::model::DrawLight;
        render_pass.set_pipeline(&self.light_render_pipeline);
        render_pass.set_vertex_buffer(1, self.instance_buffer.slice(..));
        render_pass.draw_light_model(
            &self.obj_model,
            &self.camera_bind_group,
            &self.light_bind_group,
        );

        render_pass.set_pipeline(pipeline);

        // encoder.finish needs ownership of encoder, render_pass is not needed any more and holds a ref, so drop it
        drop(render_pass);
        self.depth_pass.render(&frame, &mut encoder);
        self.queue.submit(std::iter::once(encoder.finish()));
        Ok(())
    }
}

fn main() {
    env_logger::init();
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();
    let mut state = block_on(State::new(&window));

    let mut last_render_time = std::time::Instant::now();
    event_loop.run(move |event, _, control_flow| match event {
        Event::RedrawRequested(_window_id) => {
            let now = std::time::Instant::now();
            let dt = now - last_render_time;
            last_render_time = now;
            state.update(dt);
            match state.render() {
                Ok(_) => {}
                // recreate swap_chain if lost
                Err(wgpu::SwapChainError::Lost) => state.resize(state.size),
                Err(wgpu::SwapChainError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                // all other errors (Outdated, Timeout) should be resolved by the next frame
                Err(e) => println!("{:?}", e),
            }
        }
        Event::DeviceEvent { ref event, .. } => {
            state.input(event);
        }
        Event::MainEventsCleared => {
            // all events have been handled
            window.request_redraw();
        }
        Event::WindowEvent {
            ref event,
            window_id,
        } if window_id == window.id() => {
            match event {
                WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                WindowEvent::KeyboardInput { input, .. } => match input {
                    KeyboardInput {
                        state: ElementState::Pressed,
                        virtual_keycode: Some(VirtualKeyCode::Escape),
                        ..
                    } => *control_flow = ControlFlow::Exit,
                    _ => {}
                },
                WindowEvent::Resized(physical_size) => {
                    state.resize(*physical_size);
                }
                WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                    // new_inner_size is &&mut so we have to dereference twice
                    state.resize(**new_inner_size);
                }
                _ => {}
            }
        }
        _ => {}
    });
}
