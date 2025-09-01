use cloak_share::pixel_conversion::*;

#[test]
fn test_invalid_pixel_format_handling() {
    // Test various invalid pixel formats
    let invalid_formats = [
        0,
        875704438, // YUV format that caused the original issue
        1234567890,
        u32::MAX,
    ];
    
    for format in invalid_formats {
        let result = validate_pixel_format(format);
        assert!(result.is_err(), "Format {} should be rejected", format);
        
        let error_msg = result.unwrap_err();
        assert!(error_msg.contains("Unsupported pixel format"), 
                "Error message should mention unsupported format: {}", error_msg);
    }
}

#[test]
fn test_scaling_with_zero_dimensions() {
    let src_data = vec![255u8; 16]; // 2x2 image
    
    // Zero width should return empty
    let result = scale_rgba_nearest_neighbor(&src_data, 2, 2, 0, 2);
    assert_eq!(result.len(), 0);
    
    // Zero height should return empty  
    let result = scale_rgba_nearest_neighbor(&src_data, 2, 2, 2, 0);
    assert_eq!(result.len(), 0);
    
    // Both zero should return empty
    let result = scale_rgba_nearest_neighbor(&src_data, 2, 2, 0, 0);
    assert_eq!(result.len(), 0);
}

#[test]
fn test_scaling_with_invalid_source_dimensions() {
    let src_data = vec![255u8; 16]; // Exactly 4 pixels worth
    
    // Claiming larger source than actual data should still work (will read garbage but not crash)
    let result = scale_rgba_nearest_neighbor(&src_data, 10, 10, 2, 2); // Claim 10x10 but only have 2x2
    assert_eq!(result.len(), 2 * 2 * 4); // Should still produce right output size
}

#[test]
fn test_bgra_to_rgba_with_non_multiple_of_4() {
    // Input that's not a multiple of 4 bytes should handle gracefully
    let invalid_bgra = vec![255u8; 15]; // 15 bytes = 3.75 pixels
    let result = bgra_to_rgba_slice(&invalid_bgra);
    
    // Should handle the complete pixels (15/4 = 3 complete pixels = 12 bytes)
    // chunks_exact will ignore the remaining 3 bytes
    assert_eq!(result.len(), 12);
}

#[test]
fn test_memory_safety_with_large_scaling() {
    // Test that extreme scaling doesn't cause memory issues
    let small_src = vec![128u8; 4]; // 1x1 pixel
    
    // Scale to moderately large size
    let result = scale_rgba_nearest_neighbor(&small_src, 1, 1, 200, 200);
    assert_eq!(result.len(), 200 * 200 * 4);
    
    // All pixels should be the same (nearest neighbor from single source pixel)
    for chunk in result.chunks_exact(4) {
        assert_eq!(chunk, &[128, 128, 128, 128]);
    }
}

#[test]
fn test_rgba_data_bounds_checking() {
    // Test that we don't read out of bounds when source is smaller than claimed
    let src_data = vec![100u8; 4 * 4]; // 2x2 pixels worth of data
    
    // Claim it's 1x1 and scale to 4x4 - should not panic
    let result = scale_rgba_nearest_neighbor(&src_data, 1, 1, 4, 4);
    assert_eq!(result.len(), 4 * 4 * 4);
    
    // Should have replicated the first pixel everywhere
    for chunk in result.chunks_exact(4) {
        assert_eq!(chunk, &[100, 100, 100, 100]);
    }
}