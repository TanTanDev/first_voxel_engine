use anyhow::*;
use image::GenericImageView;
use std::path::Path;
use wgpu::util::DeviceExt;

use crate::color::Color;

pub struct Texture {
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub sampler: wgpu::Sampler,
}

impl Texture {
    #[allow(dead_code)]
    pub fn from_bytes_to_image(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        bytes: &[u8],
        label: &str,
    ) -> Result<Self> {
        let img = image::load_from_memory(bytes)?;
        Self::from_image(device, queue, &img, Some(label))
    }

    pub fn from_color(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        color: Color,
        width: u32,
        height: u32,
    ) -> Result<Self> {
        let mut bytes = Vec::with_capacity((width * height) as usize);
        for _y in 0..height {
            for _x in 0..width {
                let colors: [u8; 4] = color.into();
                bytes.push(colors[0]);
                bytes.push(colors[1]);
                bytes.push(colors[2]);
                bytes.push(colors[3]);
            }
        }
        let bytes: &[u8] = bytes.as_ref();
        Self::from_bytes(device, queue, &bytes, width, height, Some("color texture"))
    }

    pub fn from_bytes(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        bytes: &[u8],
        width: u32,
        height: u32,
        label: Option<&str>,
    ) -> Result<Self> {
        let texture_size = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };

        let texture_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label,
            contents: bytes,
            usage: wgpu::BufferUsage::COPY_SRC,
        });

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("temp texture encoder"),
        });

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            size: texture_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            // sampled: use in shader
            // copy dst, we want to copy data to this texture
            usage: wgpu::TextureUsage::SAMPLED | wgpu::TextureUsage::COPY_DST,
            label: Some("my texture"),
        });
        encoder.copy_buffer_to_texture(
            wgpu::ImageCopyBuffer {
                buffer: &texture_buffer,
                layout: wgpu::ImageDataLayout {
                    offset: 0,
                    bytes_per_row: std::num::NonZeroU32::new(4 * width),
                    rows_per_image: std::num::NonZeroU32::new(height),
                },
            },
            wgpu::ImageCopyTexture {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
            },
            texture_size,
        );
        queue.submit(std::iter::once(encoder.finish()));
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let address_mode = wgpu::AddressMode::ClampToEdge;
        let filter_mode = wgpu::FilterMode::Nearest;
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: address_mode,
            address_mode_v: address_mode,
            address_mode_w: address_mode,
            mag_filter: filter_mode,
            min_filter: filter_mode,
            mipmap_filter: filter_mode,
            ..Default::default()
        });
        Ok(Self {
            texture,
            view,
            sampler,
        })
    }

    pub fn from_image(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        image: &image::DynamicImage,
        label: Option<&str>,
    ) -> Result<Self> {
        let diffuse_rgba = image.to_rgba8();
        let dimensions = image.dimensions();
        Self::from_bytes(
            device,
            queue,
            &diffuse_rgba,
            dimensions.0,
            dimensions.1,
            label,
        )
    }

    pub const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;
    pub fn create_depth_texture(
        device: &wgpu::Device,
        sc_desc: &wgpu::SwapChainDescriptor,
        label: &str,
    ) -> Self {
        let size = wgpu::Extent3d {
            width: sc_desc.width,
            height: sc_desc.height,
            depth_or_array_layers: 1,
        };
        let desc = wgpu::TextureDescriptor {
            label: Some(label),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: Self::DEPTH_FORMAT,
            usage: wgpu::TextureUsage::RENDER_ATTACHMENT | wgpu::TextureUsage::SAMPLED,
        };
        let texture = device.create_texture(&desc);
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            compare: Some(wgpu::CompareFunction::LessEqual),
            lod_min_clamp: -100.0,
            lod_max_clamp: 100.0,
            ..Default::default()
        });
        Self {
            texture,
            view,
            sampler,
        }
    }

    pub fn load<P: AsRef<Path>>(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        path: P,
    ) -> Result<Self> {
        // uuh borrow checker pls...
        let path_copy = path.as_ref().to_path_buf();
        let label = path_copy.to_str();
        println!("loading image from: {:?}", label);
        let img = image::open(path)?;
        Self::from_image(device, queue, &img, label)
        //let path_copy =
    }
}
