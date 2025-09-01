use core_foundation::base::TCFType;
use core_video_sys::{
    CVPixelBufferGetBaseAddress, CVPixelBufferGetBytesPerRow, CVPixelBufferGetHeight,
    CVPixelBufferGetPixelFormatType, CVPixelBufferGetWidth, CVPixelBufferLockBaseAddress,
    CVPixelBufferRef, CVPixelBufferUnlockBaseAddress,
    kCVPixelBufferLock_ReadOnly, kCVPixelFormatType_32BGRA,
};
use screencapturekit::output::CMSampleBuffer;

pub fn bgra_to_rgba_slice(bgra_data: &[u8]) -> Vec<u8> {
    bgra_data
        .chunks_exact(4)
        .flat_map(|bgra| [bgra[2], bgra[1], bgra[0], bgra[3]]) // Swap B and R
        .collect()
}

pub fn scale_rgba_nearest_neighbor(
    src_data: &[u8],
    src_width: usize,
    src_height: usize,
    dst_width: usize,
    dst_height: usize,
) -> Vec<u8> {
    let mut dst = vec![0u8; dst_width * dst_height * 4];
    
    // Handle zero dimensions
    if dst_width == 0 || dst_height == 0 || src_width == 0 || src_height == 0 {
        return dst;
    }
    
    // Validate source data size
    let expected_src_len = src_width * src_height * 4;
    if src_data.len() < expected_src_len {
        eprintln!("Warning: source data too small. Expected {}, got {}", expected_src_len, src_data.len());
        return dst; // Return black image
    }
    
    let scale_x = src_width as f32 / dst_width as f32;
    let scale_y = src_height as f32 / dst_height as f32;

    for y in 0..dst_height {
        let src_y = ((y as f32 * scale_y) as usize).min(src_height.saturating_sub(1));
        for x in 0..dst_width {
            let src_x = ((x as f32 * scale_x) as usize).min(src_width.saturating_sub(1));
            
            let src_idx = (src_y * src_width + src_x) * 4;
            let dst_idx = (y * dst_width + x) * 4;
            
            // Bounds check before copying
            if src_idx + 4 <= src_data.len() && dst_idx + 4 <= dst.len() {
                dst[dst_idx..dst_idx + 4].copy_from_slice(&src_data[src_idx..src_idx + 4]);
            }
        }
    }
    
    dst
}

pub fn validate_pixel_format(format: u32) -> Result<(), String> {
    if format == kCVPixelFormatType_32BGRA {
        Ok(())
    } else {
        Err(format!("Unsupported pixel format: {}", format))
    }
}

/// Converts ScreenCaptureKit CMSampleBuffer (chunky BGRA) -> RGBA 1920x1080.
/// Returns None if the buffer isn't BGRA or if locking/base address fails.
pub fn convert_sample_buffer_to_rgba(sample_buffer: &CMSampleBuffer) -> Option<Vec<u8>> {
    // 1) Get CVPixelBuffer
    let pixel_buffer = sample_buffer.get_pixel_buffer().ok()?;
    let pixel_buffer_rs = pixel_buffer.as_concrete_TypeRef(); // *mut __CVPixelBufferRef (rs)
    let pixel_buffer_ref = pixel_buffer_rs.cast(); // We cast __CVPixelBufferRef to *mut __CVBuffer (sys)

    // 2) Lock for read
    let lock_flags = kCVPixelBufferLock_ReadOnly;
    let lock_result = unsafe { CVPixelBufferLockBaseAddress(pixel_buffer_ref, lock_flags) };
    if lock_result != 0 {
        eprintln!("Failed to lock CVPixelBuffer");
        return None;
    }

    // Helper to ensure unlock on early returns
    struct Unlock<'a> {
        pb: CVPixelBufferRef,
        flags: u64,
        _m: std::marker::PhantomData<&'a ()>,
    }
    impl<'a> Drop for Unlock<'a> {
        fn drop(&mut self) {
            unsafe { CVPixelBufferUnlockBaseAddress(self.pb, self.flags) };
        }
    }
    let _unlock_guard = Unlock {
        pb: pixel_buffer_ref,
        flags: lock_flags,
        _m: std::marker::PhantomData,
    };

    // 3) Read properties
    let width = unsafe { CVPixelBufferGetWidth(pixel_buffer_ref) } as usize;
    let height = unsafe { CVPixelBufferGetHeight(pixel_buffer_ref) } as usize;
    let bytes_per_row = unsafe { CVPixelBufferGetBytesPerRow(pixel_buffer_ref) } as usize;
    let pixel_format = unsafe { CVPixelBufferGetPixelFormatType(pixel_buffer_ref) };
    
    if pixel_format != kCVPixelFormatType_32BGRA {
        eprintln!(
            "Unexpected pixel format: {}, expected kCVPixelFormatType_32BGRA",
            pixel_format
        );
        return None; // _unlock_guard will unlock
    }

    // 4) Base address -> slice
    let base_ptr = unsafe { CVPixelBufferGetBaseAddress(pixel_buffer_ref) } as *const u8;
    if base_ptr.is_null() {
        eprintln!("CVPixelBuffer base address is null");
        return None;
    }

    // Sanity check: bytes_per_row must be >= width*4 for BGRA
    let min_bpr = width.checked_mul(4)?;
    if bytes_per_row < min_bpr {
        eprintln!("bytes_per_row ({bytes_per_row}) < width*4 ({min_bpr})");
        return None;
    }

    let src_len = bytes_per_row.checked_mul(height)?;
    let src = unsafe { std::slice::from_raw_parts(base_ptr, src_len) };

    // 5) Prepare destination RGBA 1920x1080
    const TARGET_W: usize = 1920;
    const TARGET_H: usize = 1080;
    let mut dst = vec![0u8; TARGET_W * TARGET_H * 4];

    // Fast path: same size (no scaling), just swizzle BGRA -> RGBA per pixel.
    if width == TARGET_W && height == TARGET_H {
        for y in 0..TARGET_H {
            let src_row = &src[y * bytes_per_row..y * bytes_per_row + TARGET_W * 4];
            let dst_row = &mut dst[y * TARGET_W * 4..(y + 1) * TARGET_W * 4];

            // Iterate per pixel
            for x in 0..TARGET_W {
                let si = x * 4;
                let di = x * 4;
                // BGRA -> RGBA
                let b = src_row[si + 0];
                let g = src_row[si + 1];
                let r = src_row[si + 2];
                let a = src_row[si + 3];

                dst_row[di + 0] = r;
                dst_row[di + 1] = g;
                dst_row[di + 2] = b;
                dst_row[di + 3] = a;
            }
        }
        return Some(dst); // unlock via guard
    }

    // Nearest-neighbor scaling + BGRA -> RGBA swizzle
    let scale_x = width as f32 / TARGET_W as f32;
    let scale_y = height as f32 / TARGET_H as f32;

    for y in 0..TARGET_H {
        let src_y = ((y as f32 * scale_y) as usize).min(height.saturating_sub(1));
        let src_row_base = src_y * bytes_per_row;

        for x in 0..TARGET_W {
            let src_x = ((x as f32 * scale_x) as usize).min(width.saturating_sub(1));

            let si = src_row_base + src_x * 4;
            let di = (y * TARGET_W + x) * 4;

            let b = src[si + 0];
            let g = src[si + 1];
            let r = src[si + 2];
            let a = src[si + 3];

            dst[di + 0] = r;
            dst[di + 1] = g;
            dst[di + 2] = b;
            dst[di + 3] = a;
        }
    }

    Some(dst)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bgra_to_rgba_conversion() {
        // BGRA input: Blue=1, Green=2, Red=3, Alpha=4
        let bgra = vec![1, 2, 3, 4];
        let rgba = bgra_to_rgba_slice(&bgra);
        
        // Expected RGBA: Red=3, Green=2, Blue=1, Alpha=4
        assert_eq!(rgba, vec![3, 2, 1, 4]);
    }

    #[test]
    fn test_bgra_to_rgba_multiple_pixels() {
        let bgra = vec![
            10, 20, 30, 255, // Pixel 1: B=10, G=20, R=30, A=255
            40, 50, 60, 128, // Pixel 2: B=40, G=50, R=60, A=128
        ];
        let rgba = bgra_to_rgba_slice(&bgra);
        
        assert_eq!(rgba, vec![
            30, 20, 10, 255, // Pixel 1: R=30, G=20, B=10, A=255
            60, 50, 40, 128, // Pixel 2: R=60, G=50, B=40, A=128
        ]);
    }

    #[test]
    fn test_scaling_no_change() {
        // 2x2 RGBA image
        let src = vec![
            255, 0, 0, 255,   // Red pixel
            0, 255, 0, 255,   // Green pixel
            0, 0, 255, 255,   // Blue pixel
            255, 255, 0, 255, // Yellow pixel
        ];
        
        let result = scale_rgba_nearest_neighbor(&src, 2, 2, 2, 2);
        assert_eq!(result, src);
    }

    #[test]
    fn test_scaling_upscale_2x() {
        // 1x1 red pixel
        let src = vec![255, 0, 0, 255];
        
        let result = scale_rgba_nearest_neighbor(&src, 1, 1, 2, 2);
        
        // Should become 2x2 of red pixels
        let expected = vec![
            255, 0, 0, 255, 255, 0, 0, 255,
            255, 0, 0, 255, 255, 0, 0, 255,
        ];
        assert_eq!(result, expected);
    }

    #[test]
    fn test_scaling_downscale() {
        // 2x2 image -> 1x1
        let src = vec![
            255, 0, 0, 255,   // Red
            0, 255, 0, 255,   // Green
            0, 0, 255, 255,   // Blue  
            255, 255, 0, 255, // Yellow
        ];
        
        let result = scale_rgba_nearest_neighbor(&src, 2, 2, 1, 1);
        
        // Should pick top-left pixel (red)
        assert_eq!(result, vec![255, 0, 0, 255]);
    }

    #[test]
    fn test_validate_pixel_format_success() {
        assert!(validate_pixel_format(kCVPixelFormatType_32BGRA).is_ok());
    }

    #[test]
    fn test_validate_pixel_format_failure() {
        let result = validate_pixel_format(875704438); // YUV format
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Unsupported pixel format"));
    }

    #[test]
    fn test_edge_case_empty_input() {
        let result = bgra_to_rgba_slice(&[]);
        assert_eq!(result, vec![]);
    }

    #[test]
    fn test_edge_case_single_pixel() {
        let bgra = vec![100, 150, 200, 255];
        let rgba = bgra_to_rgba_slice(&bgra);
        assert_eq!(rgba, vec![200, 150, 100, 255]);
    }

    #[test]
    fn test_scaling_edge_case_zero_size() {
        let src = vec![255, 0, 0, 255];
        let result = scale_rgba_nearest_neighbor(&src, 1, 1, 0, 0);
        assert_eq!(result, vec![]);
    }
}