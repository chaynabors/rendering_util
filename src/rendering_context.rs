use log::warn;
use wgpu::Adapter;
use wgpu::Backends;
use wgpu::Device;
use wgpu::DeviceDescriptor;
use wgpu::Features;
use wgpu::Instance;
use wgpu::Limits;
use wgpu::PowerPreference;
use wgpu::PresentMode;
use wgpu::Queue;
use wgpu::RequestAdapterOptions;
use wgpu::Surface;
use wgpu::SurfaceConfiguration;
use wgpu::SurfaceError;
use wgpu::TextureUsages;
use wgpu::TextureView;
use wgpu::TextureViewDescriptor;

use crate::Error;

#[allow(dead_code)]
pub struct RenderingContext {
    instance: Instance,
    surface: Surface,
    adapter: Adapter,
    device: Device,
    queue: Queue,
    surface_conf: SurfaceConfiguration,
}

impl RenderingContext {
    pub async fn new<W: raw_window_handle::HasRawWindowHandle>(
        window: &W,
        width: u32,
        height: u32,
    ) -> Result<Self, Error> {
        let instance = Instance::new(Backends::PRIMARY);

        let surface = unsafe { instance.create_surface(&window) };

        let adapter = match instance.request_adapter(&RequestAdapterOptions {
            power_preference: PowerPreference::HighPerformance,
            force_fallback_adapter: false,
            compatible_surface: Some(&surface),
        }).await {
            Some(adapter) => adapter,
            None => return Err(Error::NoSuitableAdapter),
        };

        let (device, queue) = match adapter.request_device(
            &DeviceDescriptor {
                label: Some("device"),
                features: Features::empty(),
                limits: Limits::default()
            },
            None,
        ).await {
            Ok(dq) => dq,
            Err(_) => return Err(Error::NoSuitableDevice),
        };

        let surface_conf = SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format: match surface.get_preferred_format(&adapter) {
                Some(format) => format,
                None => return Err(Error::IncompatibleSurface),
            },
            width,
            height,
            present_mode: PresentMode::Mailbox,
        };
        surface.configure(&device, &surface_conf);

        Ok(Self {
            instance,
            surface,
            adapter,
            device,
            queue,
            surface_conf,
        })
    }

    pub fn render<F>(&mut self, width: u32, height: u32, mut handler: F) -> Result<(), Error> where
        F: FnMut(&Device, &TextureView),
    {
        if width == 0 || height == 0 { return Ok(()); }

        // Update the surface if our size changed
        if self.surface_conf.width != width || self.surface_conf.height != height {
            self.surface_conf.width = width;
            self.surface_conf.height = height;
            self.surface.configure(&self.device, &self.surface_conf);
        }

        // Retrieve our current surface texture
        let surface = match self.surface.get_current_texture() {
            Ok(frame) => frame,
            Err(e) => match e {
                SurfaceError::Timeout => {
                    warn!("Timed out while retrieving surface");
                    return Ok(());
                },
                SurfaceError::Outdated => {
                    warn!("Retrieved surface was outdated");
                    return Ok(());
                },
                SurfaceError::Lost => return Err(Error::SurfaceLost),
                SurfaceError::OutOfMemory => return Err(Error::OutOfMemory),
            },
        };

        // Build a view of our surface texture
        let surface_view = surface.texture.create_view(&TextureViewDescriptor::default());

        // Call user function
        handler(&self.device, &surface_view);

        // Finish
        surface.present();
        Ok(())
    }
}
