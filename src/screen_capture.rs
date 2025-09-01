use std::sync::{Arc, Mutex};
use screencapturekit::{
    output::CMSampleBuffer,
    shareable_content::SCShareableContent, 
    stream::{
        SCStream,
        configuration::SCStreamConfiguration,
        content_filter::SCContentFilter,
        output_trait::SCStreamOutputTrait,
        output_type::SCStreamOutputType,
        configuration::pixel_format::PixelFormat,
    },
};

pub struct ScreenCaptureManager {
    pub latest_frame: Arc<Mutex<Option<Vec<u8>>>>,
    stream: Option<SCStream>,
}

impl ScreenCaptureManager {
    pub fn new() -> Self {
        Self {
            latest_frame: Arc::new(Mutex::new(None)),
            stream: None,
        }
    }

    pub fn start_capture(&mut self) -> Result<(), String> {
        // Get shareable content + pick the main display
        let shareable = SCShareableContent::get()
            .map_err(|e| format!("Failed to get SCShareableContent: {:?}", e))?;
        
        let display = shareable
            .displays()
            .first()
            .ok_or("No displays found")?
            .clone();

        // Build a content filter for the display
        let filter = SCContentFilter::new().with_display_excluding_windows(&display, &[]);

        // Configure the stream
        let config = SCStreamConfiguration::new()
            .set_width(1920)
            .map_err(|e| format!("Failed to set width: {:?}", e))?
            .set_height(1080)
            .map_err(|e| format!("Failed to set height: {:?}", e))?
            .set_captures_audio(false)
            .map_err(|e| format!("Failed to set audio: {:?}", e))?
            .set_pixel_format(PixelFormat::BGRA)
            .map_err(|e| format!("Failed to set pixel format: {:?}", e))?;

        // Create output handler
        let output_handler = ScreenCaptureOutputHandler {
            frame_data: self.latest_frame.clone(),
        };

        // Create stream, add output, start
        let mut stream = SCStream::new(&filter, &config);
        stream.add_output_handler(output_handler, SCStreamOutputType::Screen);
        stream.start_capture()
            .map_err(|e| format!("Failed to start capture: {:?}", e))?;

        self.stream = Some(stream);
        println!("Screen capture started!");
        Ok(())
    }

    pub fn get_latest_frame(&self) -> Option<Vec<u8>> {
        self.latest_frame.lock().ok()?.clone()
    }

    pub fn stop_capture(&mut self) {
        if let Some(stream) = self.stream.take() {
            if let Err(e) = stream.stop_capture() {
                eprintln!("Failed to stop capture: {:?}", e);
            }
        }
    }
}

impl Drop for ScreenCaptureManager {
    fn drop(&mut self) {
        self.stop_capture();
    }
}

/// Output handler for ScreenCaptureKit frames
struct ScreenCaptureOutputHandler {
    frame_data: Arc<Mutex<Option<Vec<u8>>>>,
}

impl SCStreamOutputTrait for ScreenCaptureOutputHandler {
    fn did_output_sample_buffer(
        &self,
        sample_buffer: CMSampleBuffer,
        output_type: SCStreamOutputType,
    ) {
        if matches!(output_type, SCStreamOutputType::Screen) {
            if let Some(rgba_data) = crate::pixel_conversion::convert_sample_buffer_to_rgba(&sample_buffer) {
                if let Ok(mut latest) = self.frame_data.lock() {
                    *latest = Some(rgba_data);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_screen_capture_manager_creation() {
        let manager = ScreenCaptureManager::new();
        assert!(manager.latest_frame.lock().unwrap().is_none());
        assert!(manager.stream.is_none());
    }

    #[test]
    fn test_get_latest_frame_empty() {
        let manager = ScreenCaptureManager::new();
        assert!(manager.get_latest_frame().is_none());
    }

    #[test]
    fn test_frame_data_update() {
        let manager = ScreenCaptureManager::new();
        
        // Simulate frame update
        {
            let mut frame = manager.latest_frame.lock().unwrap();
            *frame = Some(vec![255u8; 1920 * 1080 * 4]);
        }
        
        let retrieved_frame = manager.get_latest_frame();
        assert!(retrieved_frame.is_some());
        assert_eq!(retrieved_frame.unwrap().len(), 1920 * 1080 * 4);
    }

    #[test]
    fn test_output_handler_creation() {
        let frame_data = Arc::new(Mutex::new(None));
        let _handler = ScreenCaptureOutputHandler {
            frame_data: frame_data.clone(),
        };
        
        // Handler should not modify frame data on creation
        assert!(frame_data.lock().unwrap().is_none());
    }
}