use cloak_share::pixel_conversion::*;
use core_video_sys::kCVPixelFormatType_32BGRA;

#[test]
fn property_test_bgra_to_rgba_preserves_pixel_count() {
    // Test with various sizes
    for size in [0, 4, 100, 1000, 10000] {
        let bgra_data = vec![128u8; size];
        let rgba_data = bgra_to_rgba_slice(&bgra_data);
        assert_eq!(rgba_data.len(), bgra_data.len(), "Size mismatch for input size {}", size);
    }
}

#[test] 
fn property_test_bgra_to_rgba_channel_swap() {
    // Test that B and R channels are always swapped, G and A preserved
    for _ in 0..100 {
        let b = rand_u8();
        let g = rand_u8(); 
        let r = rand_u8();
        let a = rand_u8();
        
        let bgra = vec![b, g, r, a];
        let rgba = bgra_to_rgba_slice(&bgra);
        
        assert_eq!(rgba[0], r, "Red channel not swapped correctly");
        assert_eq!(rgba[1], g, "Green channel not preserved"); 
        assert_eq!(rgba[2], b, "Blue channel not swapped correctly");
        assert_eq!(rgba[3], a, "Alpha channel not preserved");
    }
}

#[test]
fn property_test_scaling_preserves_total_pixels() {
    let test_cases = [
        (10, 10, 20, 20),   // 2x upscale
        (20, 20, 10, 10),   // 2x downscale  
        (15, 10, 30, 15),   // Different aspect ratio
        (1, 1, 100, 100),   // Extreme upscale
        (100, 100, 1, 1),   // Extreme downscale
    ];
    
    for (src_w, src_h, dst_w, dst_h) in test_cases {
        let src_data = vec![255u8; src_w * src_h * 4]; // White pixels
        let result = scale_rgba_nearest_neighbor(&src_data, src_w, src_h, dst_w, dst_h);
        
        let expected_len = dst_w * dst_h * 4;
        assert_eq!(result.len(), expected_len, 
                   "Scaling {}x{} -> {}x{} produced wrong output size", 
                   src_w, src_h, dst_w, dst_h);
    }
}

#[test]
fn property_test_scaling_preserves_alpha_channel() {
    // Alpha should be preserved in all scaling operations
    for _ in 0..20 {
        let src_w = (rand_u8() as usize % 50) + 1; // 1-50
        let src_h = (rand_u8() as usize % 50) + 1;
        let dst_w = (rand_u8() as usize % 50) + 1;
        let dst_h = (rand_u8() as usize % 50) + 1;
        
        // Create source with random alpha values
        let mut src_data = vec![0u8; src_w * src_h * 4];
        for i in (3..src_data.len()).step_by(4) {
            src_data[i] = rand_u8(); // Random alpha
        }
        
        let result = scale_rgba_nearest_neighbor(&src_data, src_w, src_h, dst_w, dst_h);
        
        // Check that all alpha values in result exist in source
        for alpha in result.iter().skip(3).step_by(4) {
            let alpha_exists = src_data.iter().skip(3).step_by(4).any(|&src_alpha| src_alpha == *alpha);
            assert!(alpha_exists, "Alpha value {} not found in source data", alpha);
        }
    }
}

#[test]
fn property_test_validate_pixel_format_consistency() {
    // Valid format should always pass
    for _ in 0..10 {
        assert!(validate_pixel_format(kCVPixelFormatType_32BGRA).is_ok());
    }
    
    // Invalid formats should always fail
    let invalid_formats = [0, 1, 875704438, 999999999, u32::MAX];
    for format in invalid_formats {
        if format != kCVPixelFormatType_32BGRA {
            assert!(validate_pixel_format(format).is_err(), "Format {} should be invalid", format);
        }
    }
}

#[test]
fn property_test_scaling_identity() {
    // Scaling to same size should be identity operation
    for size in [1, 5, 10, 50] {
        let src_data: Vec<u8> = (0..size * size * 4).map(|i| (i % 256) as u8).collect();
        let result = scale_rgba_nearest_neighbor(&src_data, size, size, size, size);
        assert_eq!(result, src_data, "Identity scaling failed for {}x{}", size, size);
    }
}

#[test]
fn property_test_bgra_rgba_roundtrip_stability() {
    // Converting BGRA->RGBA->BGRA should be stable (modulo channel order)
    for _ in 0..50 {
        let original_bgra = vec![rand_u8(), rand_u8(), rand_u8(), rand_u8()];
        let rgba = bgra_to_rgba_slice(&original_bgra);
        
        // Manual RGBA back to BGRA
        let back_to_bgra = vec![rgba[2], rgba[1], rgba[0], rgba[3]];
        
        assert_eq!(back_to_bgra, original_bgra, "BGRA->RGBA->BGRA roundtrip failed");
    }
}

// Simple random number generator for testing (no external deps needed)
fn rand_u8() -> u8 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    use std::time::{SystemTime, UNIX_EPOCH};
    
    let mut hasher = DefaultHasher::new();
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos().hash(&mut hasher);
    (hasher.finish() % 256) as u8
}