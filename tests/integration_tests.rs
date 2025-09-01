use std::sync::{Arc, Mutex};

#[test]
fn test_safe_mirror_initialization() {
    // Test that we can create the basic structures without panicking
    let latest_frame: Arc<Mutex<Option<Vec<u8>>>> = Arc::new(Mutex::new(None));
    
    // Simulate frame data
    let test_rgba = vec![255u8; 1920 * 1080 * 4]; // White frame
    {
        let mut frame = latest_frame.lock().unwrap();
        *frame = Some(test_rgba);
    }
    
    // Verify data was stored correctly
    let frame = latest_frame.lock().unwrap();
    assert!(frame.is_some());
    assert_eq!(frame.as_ref().unwrap().len(), 1920 * 1080 * 4);
}

#[test]
fn test_frame_data_thread_safety() {
    let latest_frame: Arc<Mutex<Option<Vec<u8>>>> = Arc::new(Mutex::new(None));
    let frame_clone = latest_frame.clone();
    
    // Simulate concurrent access like ScreenCaptureKit would do
    let handle = std::thread::spawn(move || {
        let test_data = vec![128u8; 1920 * 1080 * 4]; // Gray frame
        if let Ok(mut frame) = frame_clone.lock() {
            *frame = Some(test_data);
        }
    });
    
    handle.join().unwrap();
    
    // Verify the frame was updated by the thread
    let frame = latest_frame.lock().unwrap();
    assert!(frame.is_some());
    assert_eq!(frame.as_ref().unwrap()[0], 128);
}

#[test]
fn test_texture_data_size_validation() {
    let valid_size = 1920 * 1080 * 4;
    let test_data = vec![0u8; valid_size];
    
    // This should be exactly the right size
    assert_eq!(test_data.len(), valid_size);
    
    // Test invalid sizes
    let too_small = vec![0u8; valid_size - 1];
    let too_large = vec![0u8; valid_size + 1];
    
    assert_ne!(too_small.len(), valid_size);
    assert_ne!(too_large.len(), valid_size);
}

#[test]
fn test_rgba_data_integrity() {
    // Create a known pattern
    let mut test_data = vec![0u8; 16]; // 4 pixels worth
    
    // Pixel 1: Red
    test_data[0] = 255; test_data[1] = 0; test_data[2] = 0; test_data[3] = 255;
    // Pixel 2: Green  
    test_data[4] = 0; test_data[5] = 255; test_data[6] = 0; test_data[7] = 255;
    // Pixel 3: Blue
    test_data[8] = 0; test_data[9] = 0; test_data[10] = 255; test_data[11] = 255;
    // Pixel 4: White
    test_data[12] = 255; test_data[13] = 255; test_data[14] = 255; test_data[15] = 255;
    
    // Verify RGBA ordering is preserved
    assert_eq!(test_data[0], 255); // Red component of pixel 1
    assert_eq!(test_data[5], 255); // Green component of pixel 2
    assert_eq!(test_data[10], 255); // Blue component of pixel 3
    assert_eq!(test_data[15], 255); // Alpha component of pixel 4
}