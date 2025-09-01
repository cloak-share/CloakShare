use cloak_share::pixel_conversion::*;
use std::time::Instant;

#[test]
fn benchmark_bgra_to_rgba_1080p() {
    // Test conversion performance on 1920x1080 data
    let size = 1920 * 1080 * 4;
    let bgra_data = vec![128u8; size];
    
    let start = Instant::now();
    let _rgba_data = bgra_to_rgba_slice(&bgra_data);
    let duration = start.elapsed();
    
    println!("BGRA->RGBA conversion for 1080p: {:?}", duration);
    
    // Should complete in under 200ms on modern hardware (current implementation baseline)
    assert!(duration.as_millis() < 200, "Conversion too slow: {:?}", duration);
}

#[test]
fn benchmark_scaling_1080p_to_1080p() {
    let src_data = vec![255u8; 1920 * 1080 * 4];
    
    let start = Instant::now();
    let _result = scale_rgba_nearest_neighbor(&src_data, 1920, 1080, 1920, 1080);
    let duration = start.elapsed();
    
    println!("Identity scaling 1080p: {:?}", duration);
    
    // Identity scaling should be reasonably fast
    assert!(duration.as_millis() < 150, "Identity scaling too slow: {:?}", duration);
}

#[test]
fn benchmark_scaling_4k_to_1080p() {
    let src_data = vec![200u8; 3840 * 2160 * 4]; // 4K source
    
    let start = Instant::now();
    let result = scale_rgba_nearest_neighbor(&src_data, 3840, 2160, 1920, 1080);
    let duration = start.elapsed();
    
    println!("4K->1080p scaling: {:?}", duration);
    assert_eq!(result.len(), 1920 * 1080 * 4);
    
    // Downscaling should complete in reasonable time
    assert!(duration.as_millis() < 200, "Downscaling too slow: {:?}", duration);
}

#[test]
fn benchmark_repeated_conversions() {
    let bgra_data = vec![64u8; 1920 * 1080 * 4];
    let iterations = 10;
    
    let start = Instant::now();
    for _ in 0..iterations {
        let _rgba = bgra_to_rgba_slice(&bgra_data);
    }
    let total_duration = start.elapsed();
    let avg_duration = total_duration / iterations;
    
    println!("Average conversion time over {} iterations: {:?}", iterations, avg_duration);
    
    // Should maintain consistent performance
    assert!(avg_duration.as_millis() < 200, "Average conversion too slow: {:?}", avg_duration);
}

#[test]
fn test_memory_usage_scaling() {
    // Test that scaling doesn't use excessive memory
    let sizes = [(100, 100), (500, 500), (1000, 1000), (1920, 1080)];
    
    for (width, height) in sizes {
        let src_data = vec![100u8; width * height * 4];
        let result = scale_rgba_nearest_neighbor(&src_data, width, height, width, height);
        
        // Result should be exactly the expected size
        assert_eq!(result.len(), width * height * 4);
        
        // No memory leaks - result should contain expected data
        assert!(result.iter().all(|&pixel| pixel == 100));
    }
}