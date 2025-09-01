use std::sync::{Arc, Mutex};

/// Platform-specific screen capture capabilities
pub trait ScreenCapture {
    /// Start capturing the primary display
    fn start_capture(&mut self) -> Result<(), String>;
    
    /// Get the latest captured frame as RGBA data (1920x1080x4 bytes)
    fn get_latest_frame(&self) -> Option<Vec<u8>>;
    
    /// Stop screen capture
    fn stop_capture(&mut self);
    
    /// Get the shared frame buffer for thread-safe access
    fn get_frame_buffer(&self) -> Arc<Mutex<Option<Vec<u8>>>>;
}

/// Factory for creating platform-specific screen capture implementations
pub trait ScreenCaptureFactory {
    type Capture: ScreenCapture;
    
    /// Create a new screen capture instance
    fn create() -> Self::Capture;
}

/// Platform-specific pixel format conversion
pub trait PixelConverter {
    /// Convert platform-specific buffer to RGBA format
    fn convert_to_rgba(&self, buffer: &dyn std::any::Any) -> Option<Vec<u8>>;
}

/// Supported platforms
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Platform {
    MacOS,
    Windows, 
    Linux,
}

impl Platform {
    /// Get the current platform
    pub fn current() -> Self {
        #[cfg(target_os = "macos")]
        return Platform::MacOS;
        
        #[cfg(target_os = "windows")]
        return Platform::Windows;
        
        #[cfg(target_os = "linux")]
        return Platform::Linux;
    }
    
    /// Check if the platform is supported
    pub fn is_supported(&self) -> bool {
        match self {
            Platform::MacOS => true,
            Platform::Windows => false, // TODO: Implement Windows support
            Platform::Linux => false,   // TODO: Implement Linux support
        }
    }
}