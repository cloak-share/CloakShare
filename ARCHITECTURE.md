# CloakShare Architecture

## Project Structure

```
src/
├── main.rs                    # App entry point and window event handling (122 lines)
├── lib.rs                     # Module exports
├── safe_mirror.rs             # Coordinates GPU + screen capture (53 lines)
├── gpu_renderer.rs            # wgpu/Metal GPU operations (370 lines)
├── pixel_conversion.rs        # CMSampleBuffer → RGBA conversion (131 lines)
├── cross_platform_capture.rs  # Cross-platform screen capture API (81 lines)
├── platform_detector.rs      # Platform detection and requirements (70 lines)
├── screen_capture.rs          # Legacy ScreenCaptureManager (108 lines)
└── platform/
    ├── mod.rs                 # Platform module exports
    ├── traits.rs              # Cross-platform traits
    ├── macos.rs               # macOS ScreenCaptureKit implementation
    ├── windows.rs             # Windows placeholder (DXGI planned)
    └── linux.rs               # Linux placeholder (X11/Wayland planned)
```

## Component Responsibilities

### Core Components
- **main.rs**: Application lifecycle, window events, continuous redraw loop
- **safe_mirror.rs**: High-level coordinator between GPU rendering and screen capture
- **gpu_renderer.rs**: Complete wgpu/Metal rendering pipeline setup and execution

### Platform Abstraction
- **platform/traits.rs**: Cross-platform interfaces (`ScreenCapture`, `PixelConverter`)
- **cross_platform_capture.rs**: Runtime platform selection and unified API
- **platform_detector.rs**: Platform capability checking and user guidance

### Platform Implementations
- **platform/macos.rs**: ScreenCaptureKit integration (functional)
- **platform/windows.rs**: DXGI Desktop Duplication stub (returns errors)
- **platform/linux.rs**: X11/Wayland capture stub (returns errors)

### Data Processing
- **pixel_conversion.rs**: Core Video CMSampleBuffer → RGBA 1920x1080 conversion

## Key Design Patterns

### Platform Abstraction Pattern
```rust
// Trait definition
pub trait ScreenCapture {
    fn start_capture(&mut self) -> Result<(), String>;
    fn get_latest_frame(&self) -> Option<Vec<u8>>;
}

// Platform-specific implementation
impl ScreenCapture for MacOSScreenCapture { /* ... */ }

// Cross-platform usage
let capture = CrossPlatformScreenCapture::new()?;
```

### Graceful Degradation
- Unsupported platforms show helpful error messages
- macOS without permissions falls back to test patterns
- GPU errors are handled gracefully with surface recreation

### Modular Architecture Benefits
1. **Separation of Concerns**: Each module has a single responsibility
2. **Platform Extensibility**: Easy to add Windows/Linux support
3. **Testability**: Each component can be tested independently
4. **Maintainability**: Clear interfaces between components

## Data Flow

```
ScreenCaptureKit → CMSampleBuffer → convert_sample_buffer_to_rgba() 
     ↓
RGBA Vec<u8> → CrossPlatformScreenCapture → SafeMirror
     ↓
GpuRenderer.update_texture() → wgpu Texture → Shader → Window
```

## Future Platform Extensions

### Windows Implementation
- Replace `platform/windows.rs` stubs with DXGI Desktop Duplication
- Add Windows-specific pixel format handling
- Update `Cargo.toml` with Windows dependencies

### Linux Implementation  
- Replace `platform/linux.rs` stubs with X11/Wayland capture
- Add Linux-specific pixel format handling
- Handle multiple display servers (X11 vs Wayland)