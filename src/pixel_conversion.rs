use core_foundation::base::TCFType;
use core_video_sys::{
    CVPixelBufferGetBaseAddress, CVPixelBufferGetBytesPerRow, CVPixelBufferGetHeight,
    CVPixelBufferGetPixelFormatType, CVPixelBufferGetWidth, CVPixelBufferLockBaseAddress,
    CVPixelBufferRef, CVPixelBufferUnlockBaseAddress, kCVPixelBufferLock_ReadOnly,
    kCVPixelFormatType_32BGRA,
};
use screencapturekit::output::CMSampleBuffer;

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
    println!("{pixel_format}");
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
