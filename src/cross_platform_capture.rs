use crate::platform::{Platform, ScreenCapture, PixelConverter};
use std::sync::{Arc, Mutex};

/// Cross-platform screen capture manager that abstracts over platform-specific implementations
pub struct CrossPlatformScreenCapture {
    capture: Box<dyn ScreenCapture>,
    converter: Box<dyn PixelConverter>,
    platform: Platform,
}

impl CrossPlatformScreenCapture {
    /// Create a new cross-platform screen capture instance
    pub fn new() -> Result<Self, String> {
        let platform = Platform::current();
        
        if !platform.is_supported() {
            return Err(format!("Platform {:?} is not yet supported", platform));
        }
        
        let (capture, converter): (Box<dyn ScreenCapture>, Box<dyn PixelConverter>) = match platform {
            Platform::MacOS => {
                #[cfg(target_os = "macos")]
                {
                    use crate::platform::macos::{MacOSScreenCaptureFactory, MacOSPixelConverter};
                    use crate::platform::ScreenCaptureFactory;
                    (
                        Box::new(MacOSScreenCaptureFactory::create()),
                        Box::new(MacOSPixelConverter)
                    )
                }
                #[cfg(not(target_os = "macos"))]
                return Err("macOS platform code not available on this system".to_string());
            }
            
            Platform::Windows => {
                #[cfg(target_os = "windows")]
                {
                    use crate::platform::windows::{WindowsScreenCaptureFactory, WindowsPixelConverter};
                    use crate::platform::ScreenCaptureFactory;
                    (
                        Box::new(WindowsScreenCaptureFactory::create()),
                        Box::new(WindowsPixelConverter)
                    )
                }
                #[cfg(not(target_os = "windows"))]
                return Err("Windows platform code not available on this system".to_string());
            }
            
            Platform::Linux => {
                #[cfg(target_os = "linux")]
                {
                    use crate::platform::linux::{LinuxScreenCaptureFactory, LinuxPixelConverter};
                    use crate::platform::ScreenCaptureFactory;
                    (
                        Box::new(LinuxScreenCaptureFactory::create()),
                        Box::new(LinuxPixelConverter)
                    )
                }
                #[cfg(not(target_os = "linux"))]
                return Err("Linux platform code not available on this system".to_string());
            }
        };
        
        Ok(Self { capture, converter, platform })
    }
    
    /// Start capturing the screen
    pub fn start_capture(&mut self) -> Result<(), String> {
        self.capture.start_capture()
    }
    
    /// Get the latest captured frame
    pub fn get_latest_frame(&self) -> Option<Vec<u8>> {
        self.capture.get_latest_frame()
    }
    
    /// Stop screen capture
    pub fn stop_capture(&mut self) {
        self.capture.stop_capture()
    }
    
    /// Get the current platform
    pub fn platform(&self) -> Platform {
        self.platform
    }
    
    /// Get frame buffer for direct access (useful for testing)
    pub fn get_frame_buffer(&self) -> Arc<Mutex<Option<Vec<u8>>>> {
        self.capture.get_frame_buffer()
    }
    
    /// Get the pixel converter for manual conversions
    pub fn converter(&self) -> &dyn PixelConverter {
        self.converter.as_ref()
    }
}