use crate::platform::Platform;

/// Utility for detecting platform capabilities and providing user-friendly messages
pub struct PlatformDetector;

impl PlatformDetector {
    /// Check if the current platform is supported and provide helpful message
    pub fn check_support() -> Result<Platform, String> {
        let platform = Platform::current();
        
        match platform {
            Platform::MacOS => {
                #[cfg(target_os = "macos")]
                {
                    Ok(platform)
                }
                #[cfg(not(target_os = "macos"))]
                {
                    Err("Internal error: detected macOS but macOS code not compiled".to_string())
                }
            }
            
            Platform::Windows => {
                Err(format!(
                    "Windows support is not implemented yet.\n\
                    To add Windows support:\n\
                    1. Implement Windows-specific screen capture using DXGI or GDI+\n\
                    2. Add pixel format conversion for Windows capture formats\n\
                    3. Update platform::windows module with real implementation"
                ))
            }
            
            Platform::Linux => {
                Err(format!(
                    "Linux support is not implemented yet.\n\
                    To add Linux support:\n\
                    1. Implement X11/Wayland screen capture using xrandr or wlroots\n\
                    2. Add pixel format conversion for Linux capture formats\n\
                    3. Update platform::linux module with real implementation"
                ))
            }
        }
    }
    
    /// Get platform-specific installation requirements
    pub fn get_requirements(platform: Platform) -> &'static str {
        match platform {
            Platform::MacOS => {
                "macOS Requirements:\n\
                - macOS 11.0+ (for ScreenCaptureKit)\n\
                - Screen Recording permission in System Preferences\n\
                - Metal-capable GPU (all modern Macs)"
            }
            
            Platform::Windows => {
                "Windows Requirements (when implemented):\n\
                - Windows 10 1903+ (for DXGI Desktop Duplication)\n\
                - DirectX 11+ compatible GPU\n\
                - Desktop Window Manager enabled"
            }
            
            Platform::Linux => {
                "Linux Requirements (when implemented):\n\
                - X11 or Wayland display server\n\
                - xrandr (X11) or wlroots protocols (Wayland)\n\
                - OpenGL 3.3+ or Vulkan capable GPU"
            }
        }
    }
}