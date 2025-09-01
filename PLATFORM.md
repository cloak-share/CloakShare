# Platform Support Architecture

CloakShare is designed with a cross-platform architecture to support multiple operating systems.

## Current Status

| Platform | Status | Implementation |
|----------|--------|----------------|
| **macOS** | ✅ Implemented | ScreenCaptureKit + Core Video |
| **Windows** | ❌ Placeholder | DXGI Desktop Duplication (planned) |
| **Linux** | ❌ Placeholder | X11/Wayland capture (planned) |

## Architecture

### Core Abstraction Layer
- `platform/traits.rs` - Defines common interfaces for all platforms
- `cross_platform_capture.rs` - Unified API that selects platform implementation
- `platform_detector.rs` - Runtime platform detection and capability checking

### Platform-Specific Implementations

#### macOS (`platform/macos.rs`)
- **Screen Capture**: ScreenCaptureKit framework
- **Pixel Format**: Core Video CMSampleBuffer → RGBA conversion
- **Requirements**: macOS 11.0+, Screen Recording permission

#### Windows (`platform/windows.rs`) - Placeholder
- **Planned**: DXGI Desktop Duplication API
- **Pixel Format**: DirectX surface → RGBA conversion
- **Requirements**: Windows 10 1903+, DirectX 11+

#### Linux (`platform/linux.rs`) - Placeholder  
- **Planned**: X11 (xrandr) or Wayland (wlroots) capture
- **Pixel Format**: X11/Wayland buffer → RGBA conversion
- **Requirements**: X11/Wayland, xrandr/wlroots

## Adding New Platform Support

To add support for a new platform:

1. **Implement the trait** in `platform/{platform}.rs`:
   ```rust
   impl ScreenCapture for NewPlatformCapture {
       fn start_capture(&mut self) -> Result<(), String> { /* ... */ }
       fn get_latest_frame(&self) -> Option<Vec<u8>> { /* ... */ }
       fn stop_capture(&mut self) { /* ... */ }
       fn get_frame_buffer(&self) -> Arc<Mutex<Option<Vec<u8>>>> { /* ... */ }
   }
   ```

2. **Update platform detection** in `platform/traits.rs`:
   - Add platform to `Platform` enum
   - Update `current()` and `is_supported()` methods

3. **Update cross-platform factory** in `cross_platform_capture.rs`:
   - Add match arm for new platform
   - Handle conditional compilation with `#[cfg]`

4. **Add platform-specific dependencies** to `Cargo.toml`:
   ```toml
   [target.'cfg(target_os = "windows")'.dependencies]
   windows = "0.51"
   ```

## Usage

The application automatically detects the platform and uses the appropriate implementation:

```rust
// Automatic platform detection
let screen_capture = CrossPlatformScreenCapture::new()?;

// All platforms use the same API
screen_capture.start_capture()?;
let frame = screen_capture.get_latest_frame();
```

## Testing on Unsupported Platforms

On Windows/Linux, the application will show:
```
✗ Platform not supported:
Windows/Linux support is not implemented yet.
To add Windows/Linux support:
1. Implement platform-specific screen capture...
2. Add pixel format conversion...
3. Update platform module...
```