use std::sync::Arc;
use winit::window::Window;

/// GPU renderer that handles all wgpu operations for screen mirroring
pub struct GpuRenderer {
    pub surface: wgpu::Surface<'static>,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub config: wgpu::SurfaceConfiguration,
    pub size: winit::dpi::PhysicalSize<u32>,
    pub render_pipeline: wgpu::RenderPipeline,
    pub bind_group: wgpu::BindGroup,
    pub texture: wgpu::Texture,
}

impl GpuRenderer {
    pub async fn new(window: Arc<Window>) -> Self {
        let size = window.inner_size();

        // STEP 1: Create wgpu instance - this is our entry point to GPU programming
        // wgpu is a Rust library that provides safe access to GPU APIs (Metal, Vulkan, DirectX)
        // We specify Metal backend because we're on macOS and want direct access to Apple's GPU API
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::METAL, // Use Apple's Metal API for best macOS performance
            ..Default::default()
        });

        // STEP 2: Create surface - this connects our GPU rendering to the actual window
        // The surface is where our final rendered pixels will appear
        // Think of it as the "screen" that the GPU draws onto
        let surface = instance.create_surface(window.clone()).unwrap();

        // STEP 3: Request adapter - this finds the best GPU for our needs
        // An adapter represents a physical GPU device on the system
        // We ask for high performance GPU (discrete if available, integrated otherwise)
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance, // Prefer faster GPU over power saving
                compatible_surface: Some(&surface), // Must be able to draw to our window
                force_fallback_adapter: false,      // Don't force software rendering
            })
            .await
            .unwrap();

        // STEP 4: Request device and queue from the adapter
        // Device: Our handle to the GPU for creating resources (textures, shaders, etc.)
        // Queue: Where we submit commands to be executed by the GPU
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                required_features: wgpu::Features::empty(), // No special GPU features needed
                required_limits: wgpu::Limits::default(),   // Use standard GPU limits
                label: None,                                // Optional debug name
                memory_hints: wgpu::MemoryHints::Performance,
                trace: wgpu::Trace::Off,
            })
            .await
            .unwrap();

        // STEP 5: Configure the surface for drawing
        // Get capabilities: What color formats, present modes the GPU supports
        let surface_caps = surface.get_capabilities(&adapter);

        // Choose sRGB color format if available (standard for displays)
        // sRGB ensures colors look correct on most monitors
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb()) // Prefer sRGB for correct color display
            .unwrap_or(surface_caps.formats[0]); // Fallback to first available format

        // Surface configuration: How the GPU should draw to our window
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT, // We'll draw directly to this surface
            format: surface_format,                        // Color format (usually RGBA or BGRA)
            width: size.width,                             // Window width in pixels
            height: size.height,                           // Window height in pixels
            present_mode: surface_caps.present_modes[0],   // How to sync with display refresh
            alpha_mode: surface_caps.alpha_modes[0],       // How to handle transparency
            view_formats: vec![],                          // Additional formats (none needed)
            desired_maximum_frame_latency: 2,              // Buffer 2 frames max for responsiveness
        };
        surface.configure(&device, &config);

        // STEP 6: Create texture to hold screen capture data
        // This is GPU memory where we'll store the captured screen image
        // Think of this as a bitmap/image that lives on the GPU
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            size: wgpu::Extent3d {
                width: 1920, // Fixed resolution for now (will be dynamic later)
                height: 1080,
                depth_or_array_layers: 1, // 2D texture (not 3D or array)
            },
            mip_level_count: 1, // No mipmaps (smaller versions for distance rendering)
            sample_count: 1,    // No anti-aliasing
            dimension: wgpu::TextureDimension::D2, // 2D texture (has width and height)
            format: wgpu::TextureFormat::Rgba8UnormSrgb, // 8-bit RGBA in sRGB color space
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            // TEXTURE_BINDING: Shaders can read from this texture
            // COPY_DST: We can write screen capture data into this texture
            label: Some("Screen Capture Texture"), // Debug name
            view_formats: &[],                     // No additional view formats needed
        });

        // STEP 7: Create texture view - this is how shaders access the texture
        // A "view" is like a window into the texture data that shaders can read from
        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        // STEP 8: Create sampler - controls how the GPU reads pixels from the texture
        // When the shader asks for a pixel, the sampler decides how to interpolate/filter
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            // Address modes: What happens when texture coordinates go outside [0,1] range
            address_mode_u: wgpu::AddressMode::ClampToEdge, // Clamp to edge pixels horizontally
            address_mode_v: wgpu::AddressMode::ClampToEdge, // Clamp to edge pixels vertically
            address_mode_w: wgpu::AddressMode::ClampToEdge, // Not used for 2D textures

            // Filter modes: How to blend pixels when scaling
            mag_filter: wgpu::FilterMode::Linear, // Smooth when zooming in (linear interpolation)
            min_filter: wgpu::FilterMode::Nearest, // Crisp when zooming out (pick nearest pixel)
            mipmap_filter: wgpu::FilterMode::Nearest, // No mipmaps, so this doesn't matter
            ..Default::default()
        });

        // STEP 9: Create bind group layout - defines what resources shaders can access
        // This is like declaring the "interface" between CPU and GPU
        // We're saying: "shaders will have access to 1 texture and 1 sampler"
        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    // Binding 0: The texture containing screen capture data
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,                               // This maps to @binding(0) in the shader
                        visibility: wgpu::ShaderStages::FRAGMENT, // Only fragment shader needs this
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,                            // Not using anti-aliasing
                            view_dimension: wgpu::TextureViewDimension::D2, // 2D texture
                            sample_type: wgpu::TextureSampleType::Float { filterable: true }, // Can interpolate
                        },
                        count: None, // Single texture, not an array
                    },
                    // Binding 1: The sampler that controls how to read the texture
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,                               // This maps to @binding(1) in the shader
                        visibility: wgpu::ShaderStages::FRAGMENT, // Only fragment shader needs this
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering), // Can filter/interpolate
                        count: None, // Single sampler
                    },
                ],
                label: Some("texture_bind_group_layout"),
            });

        // STEP 10: Create the actual bind group - connects real resources to the layout
        // This binds our actual texture and sampler to the slots defined in the layout
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &texture_bind_group_layout, // Use the layout we just defined
            entries: &[
                // Bind our texture view to slot 0
                wgpu::BindGroupEntry {
                    binding: 0, // Corresponds to @binding(0) in shader
                    resource: wgpu::BindingResource::TextureView(&texture_view),
                },
                // Bind our sampler to slot 1
                wgpu::BindGroupEntry {
                    binding: 1, // Corresponds to @binding(1) in shader
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
            label: Some("texture_bind_group"),
        });

        // STEP 11: Load and compile shaders
        // Shaders are small programs that run on the GPU
        // - Vertex shader: Positions geometry (where to draw)
        // - Fragment shader: Colors pixels (what color each pixel should be)
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Screen Mirror Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        });

        // STEP 12: Create pipeline layout - defines the "interface" for the entire pipeline
        // This tells the GPU what resources (bind groups) the pipeline will use
        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[&texture_bind_group_layout], // Our texture+sampler bind group
                push_constant_ranges: &[], // No push constants (small data passed to shaders)
            });

        // STEP 13: Create the render pipeline - the complete drawing program
        // This combines vertex shader, fragment shader, and all settings into one object
        // The pipeline defines the ENTIRE process of turning data into pixels
        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),

            // VERTEX STAGE: Handles positioning and geometry
            // In our case, we create a fullscreen triangle (single large triangle)
            vertex: wgpu::VertexState {
                module: &shader,              // Use our compiled shader
                entry_point: Some("vs_main"), // Function name in shader.wgsl
                buffers: &[],                 // No vertex buffers (we generate positions in shader)
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },

            // FRAGMENT STAGE: Handles pixel coloring
            // For each pixel, this stage decides what color it should be
            fragment: Some(wgpu::FragmentState {
                module: &shader,              // Use our compiled shader
                entry_point: Some("fs_main"), // Function name in shader.wgsl
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,                  // Must match surface format
                    blend: Some(wgpu::BlendState::REPLACE), // Don't blend, just replace pixels
                    write_mask: wgpu::ColorWrites::ALL,     // Write to all color channels (RGBA)
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),

            // PRIMITIVE SETTINGS: How to interpret vertex data
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList, // Draw triangles
                strip_index_format: None,                        // Not using indexed triangles
                front_face: wgpu::FrontFace::Ccw, // Counter-clockwise triangles face forward
                cull_mode: Some(wgpu::Face::Back), // Don't draw back-facing triangles
                polygon_mode: wgpu::PolygonMode::Fill, // Fill triangles (not wireframe)
                unclipped_depth: false,           // Use normal depth clipping
                conservative: false,              // No conservative rasterization
            },

            // DEPTH/STENCIL: Not needed for 2D screen mirroring
            depth_stencil: None,

            // MULTISAMPLING: Anti-aliasing settings (disabled for performance)
            multisample: wgpu::MultisampleState {
                count: 1,                         // No multisampling
                mask: !0,                         // All samples enabled
                alpha_to_coverage_enabled: false, // No alpha-to-coverage
            },

            // MULTIVIEW: For VR/stereo rendering (not needed)
            multiview: None,
            cache: None,
        });

        Self {
            surface,
            device,
            queue,
            config,
            size,
            render_pipeline,
            bind_group,
            texture,
        }
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
        }
    }

    pub fn size(&self) -> winit::dpi::PhysicalSize<u32> {
        self.size
    }

    pub fn update_texture(&self, texture_data: &[u8]) {
        self.queue.write_texture(
            self.texture.as_image_copy(),
            texture_data,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(1920 * 4),
                rows_per_image: Some(1080),
            },
            wgpu::Extent3d {
                width: 1920,
                height: 1080,
                depth_or_array_layers: 1,
            },
        );
    }

    /// Renders one frame to the screen
    ///
    /// THE RENDERING PROCESS:
    /// 1. Get the next frame buffer from the surface (where pixels will go)
    /// 2. Create command encoder (records GPU commands)
    /// 3. Begin render pass (actual drawing operations)
    /// 4. Set pipeline and resources
    /// 5. Draw geometry (our fullscreen quad)
    /// 6. Submit commands to GPU
    /// 7. Present frame to screen
    pub fn render(&self) -> Result<(), wgpu::SurfaceError> {
        // STEP 1: Get the next frame buffer to draw into
        // This is the actual memory where our pixels will go
        let output = self.surface.get_current_texture()?;

        // Create a view of the frame buffer for rendering
        // Views define how we want to interpret the texture data
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        // STEP 2: Create command encoder - this records GPU commands
        // Instead of executing immediately, we record commands and submit them all at once
        // This is more efficient and allows the GPU to optimize execution
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        // STEP 3: Begin render pass - the actual drawing phase
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),

                // Color attachments: Where we draw pixels (the screen)
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view, // Draw to our frame buffer
                    depth_slice: None,
                    resolve_target: None, // No multisampling, so no resolve needed
                    ops: wgpu::Operations {
                        // Clear the screen to dark blue before drawing
                        // This ensures we start with a known background color
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1, // Dark red
                            g: 0.2, // Dark green
                            b: 0.3, // Dark blue
                            a: 1.0, // Fully opaque
                        }),
                        store: wgpu::StoreOp::Store, // Save the results to memory
                    },
                })],

                // We don't need depth testing for 2D screen mirroring
                depth_stencil_attachment: None,
                occlusion_query_set: None, // Not measuring occlusion
                timestamp_writes: None,    // Not measuring GPU timing
            });

            // STEP 4: Set up the render pass for drawing
            render_pass.set_pipeline(&self.render_pipeline); // Use our screen mirror pipeline
            render_pass.set_bind_group(0, &self.bind_group, &[]); // Bind texture+sampler

            // STEP 5: Draw the geometry
            // draw(vertices, instances) - we draw 3 vertices (1 large triangle), 1 instance
            // The vertex shader generates positions for a fullscreen triangle
            // Single triangle covers entire screen (fullscreen triangle trick)
            render_pass.draw(0..3, 0..1);
        } // render_pass is automatically ended here

        // STEP 6: Submit commands to GPU for execution
        // All the commands we recorded are sent to GPU as a batch
        self.queue.submit(std::iter::once(encoder.finish()));

        // STEP 7: Present the frame to the screen
        // This makes our rendered pixels visible in the window
        output.present();

        Ok(())
    }

    pub fn create_test_pattern(&self) -> Vec<u8> {
        vec![64u8; 1920 * 1080 * 4] // Dark gray fallback
    }
}
