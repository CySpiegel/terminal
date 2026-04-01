use thiserror::Error;

/// The wgpu render pipeline for terminal cell rendering.
///
/// Manages shader modules, bind group layouts, and render pipeline state.
/// Each render thread owns one `RenderPipeline` instance.
pub struct RenderPipeline {
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub surface: wgpu::Surface<'static>,
    pub config: wgpu::SurfaceConfiguration,
    cell_pipeline: Option<wgpu::RenderPipeline>,
    cursor_pipeline: Option<wgpu::RenderPipeline>,
}

impl RenderPipeline {
    /// Create a new render pipeline for the given window surface.
    pub async fn new(
        instance: &wgpu::Instance,
        surface: wgpu::Surface<'static>,
        width: u32,
        height: u32,
    ) -> Result<Self, RenderError> {
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .ok_or(RenderError::NoAdapter)?;

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: Some("terminal-device"),
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
                ..Default::default()
            }, None)
            .await
            .map_err(|e| RenderError::DeviceRequest(e.to_string()))?;

        let surface_caps = surface.get_capabilities(&adapter);
        let format = surface_caps
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width,
            height,
            present_mode: wgpu::PresentMode::Fifo, // VSync
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        Ok(Self {
            device,
            queue,
            surface,
            config,
            cell_pipeline: None,
            cursor_pipeline: None,
        })
    }

    /// Resize the surface.
    pub fn resize(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            self.config.width = width;
            self.config.height = height;
            self.surface.configure(&self.device, &self.config);
        }
    }

    /// Render a frame. Returns true if a frame was presented.
    pub fn render(&mut self) -> Result<bool, RenderError> {
        let output = self
            .surface
            .get_current_texture()
            .map_err(|_| RenderError::SurfaceLost)?;

        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("terminal-encoder"),
            });

        {
            let _render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("terminal-render-pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.0,
                            g: 0.0,
                            b: 0.0,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            // TODO: Bind cell pipeline, set vertex/instance buffers, draw
            // TODO: Bind cursor pipeline, draw cursor
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(true)
    }
}

#[derive(Debug, Error)]
pub enum RenderError {
    #[error("no suitable GPU adapter found")]
    NoAdapter,
    #[error("failed to request GPU device: {0}")]
    DeviceRequest(String),
    #[error("wgpu surface lost")]
    SurfaceLost,
    #[error("glyph rasterization failed for U+{codepoint:04X}")]
    GlyphRasterization { codepoint: u32 },
    #[error("glyph atlas full")]
    AtlasFull,
}
