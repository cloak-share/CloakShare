# CloakShare Testing Documentation

## Test Coverage Overview

We have comprehensive test coverage for the CloakShare screen capture application:

- **37 total tests** across 5 categories
- **Unit tests**: Core pixel conversion logic
- **Integration tests**: Component interaction and thread safety  
- **Property-based tests**: Invariant checking with randomized inputs
- **Performance tests**: Timing benchmarks for optimization tracking
- **Error handling tests**: Edge cases and failure modes

## Running Tests

```bash
# Run all tests
cargo test

# Run tests in release mode (for accurate performance timing)
cargo test --release

# Run specific test categories
cargo test unit
cargo test integration  
cargo test property
cargo test performance
cargo test error_handling
```

## Test Categories

### 1. Unit Tests (`src/pixel_conversion.rs`)
- ✅ BGRA to RGBA conversion correctness
- ✅ Nearest-neighbor scaling algorithms
- ✅ Pixel format validation
- ✅ Edge cases (empty data, single pixels)

### 2. Integration Tests (`tests/integration_tests.rs`)
- ✅ SafeMirror initialization without panics
- ✅ Thread safety for frame data sharing
- ✅ Texture data size validation
- ✅ RGBA data integrity preservation

### 3. Property-Based Tests (`tests/property_tests.rs`) 
- ✅ Pixel count preservation across conversions
- ✅ Channel swapping correctness (B↔R swap, G/A preserved)
- ✅ Scaling output size consistency
- ✅ Alpha channel preservation during scaling
- ✅ Identity scaling stability
- ✅ Format validation consistency

### 4. Performance Tests (`tests/performance_tests.rs`)
- ✅ 1080p BGRA→RGBA conversion: ~150ms baseline
- ✅ Identity scaling (1080p→1080p): ~100ms
- ✅ 4K→1080p downscaling: ~100ms  
- ✅ Repeated conversion consistency
- ✅ Memory usage validation for various resolutions

### 5. Error Handling Tests (`tests/error_handling_tests.rs`)
- ✅ Invalid pixel format rejection (YUV 875704438, etc.)
- ✅ Zero dimension handling
- ✅ Source data size mismatches
- ✅ Non-multiple-of-4 byte arrays
- ✅ Bounds checking for extreme scaling
- ✅ Memory safety with large operations

## Performance Baselines

Current performance baselines on modern hardware:
- **BGRA→RGBA conversion**: ~150ms for 1920×1080 
- **Nearest-neighbor scaling**: ~100ms for identity or downscaling
- **Memory usage**: Linear with pixel count, no leaks detected

## Future Test Improvements

1. **Mock ScreenCaptureKit**: Create fake CMSampleBuffer for isolated testing
2. **Visual regression**: Compare rendered output against reference images  
3. **End-to-end**: Automated screen sharing workflow validation
4. **Fuzzing**: Random pixel data and dimension combinations
5. **Performance optimization**: SIMD/GPU-accelerated pixel conversion benchmarks

## CI Integration

All tests should pass before merging:
```bash
cargo test --release && echo "✅ All tests passed"
```

Performance regressions can be detected by monitoring benchmark output over time.