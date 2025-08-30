// WGSL SHADER EXPLAINED:
// 
// Shaders are small programs that run on the GPU in parallel
// There are two types we use:
// 1. Vertex Shader: Decides WHERE to draw (positions geometry)
// 2. Fragment Shader: Decides WHAT COLOR each pixel should be
//
// Our goal: Draw a fullscreen rectangle that displays our screen capture texture

// =============================================================================
// VERTEX SHADER: Positions geometry on the screen
// =============================================================================

/// Vertex shader entry point
/// 
/// WHAT THIS DOES:
/// Creates a single large triangle that covers the entire screen
/// This is the "fullscreen triangle trick" - more efficient than using 2 triangles (quad)
/// We don't send vertex data from CPU - instead we generate positions here using vertex_index
/// 
/// WHY GENERATE POSITIONS IN SHADER:
/// - More efficient than uploading vertex buffer from CPU
/// - No memory bandwidth wasted on vertex data
/// - Simple procedural geometry generation
/// 
/// WHY ONE TRIANGLE INSTEAD OF TWO:
/// - 50% fewer vertices to process (3 instead of 6)
/// - GPU clips the oversized triangle to screen bounds automatically
/// - Industry standard technique for fullscreen effects
// Vertex output structure - passes data from vertex shader to fragment shader
struct VertexOutput {
    @builtin(position) position: vec4<f32>,  // Required: vertex position in NDC
    @location(0) tex_coords: vec2<f32>,      // Custom: texture coordinates for sampling
}

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    // COORDINATE SYSTEM EXPLANATION:
    // GPU uses "NDC" (Normalized Device Coordinates):
    // - X: -1.0 (left edge) to +1.0 (right edge)  
    // - Y: -1.0 (bottom edge) to +1.0 (top edge)
    // - Z: 0.0 (near) to 1.0 (far) - we use 0.0 for 2D
    //
    // FULLSCREEN TRIANGLE TECHNIQUE:
    // We create ONE large triangle with these 3 vertices:
    // - Vertex 0: (-1, -1) bottom-left corner of screen
    // - Vertex 1: (-1,  3) way above top-left (extends beyond screen)
    // - Vertex 2: ( 3, -1) way past bottom-right (extends beyond screen)
    //
    // The triangle is much larger than the screen, but GPU automatically clips
    // anything outside [-1,1] range, leaving us with a perfect fullscreen coverage

    // Generate the 3 vertex positions using simple math
    // This creates our oversized triangle vertices:
    //
    // vertex_index | x calculation | y calculation | final position | what it represents | winding
    // 0           | -1.0          | -1.0          | (-1, -1)       | bottom-left corner   | ↗
    // 1           | 3.0           | -1.0          | ( 3, -1)       | bottom-right (ext)   | ↑ CCW
    // 2           | -1.0          | 3.0           | (-1,  3)       | top-left (ext)       | ↘
    
    // Generate vertices in COUNTER-CLOCKWISE order (CCW) to match front_face setting
    // CCW order: 0 → 1 → 2 goes counter-clockwise when viewed from front
    var x: f32;
    var y: f32;
    
    if (vertex_index == 0u) {
        x = -1.0; y = -1.0;  // Bottom-left
    } else if (vertex_index == 1u) {
        x = 3.0; y = -1.0;   // Bottom-right (extended)
    } else {
        x = -1.0; y = 3.0;   // Top-left (extended)
    }
    
    // Calculate texture coordinates (0.0 to 1.0) from NDC coordinates (-1.0 to 1.0)
    // NDC: -1 to +1 -> Texture: 0 to 1
    // Formula: tex = (ndc + 1.0) / 2.0
    // NOTE: Y coordinate might need flipping depending on coordinate system
    let tex_x = (x + 1.0) / 2.0;
    let tex_y = 1.0 - (y + 1.0) / 2.0;  // Flip Y coordinate
    
    // Return both position and texture coordinates
    return VertexOutput(
        vec4<f32>(x, y, 0.0, 1.0),      // Position in NDC space
        vec2<f32>(tex_x, tex_y)         // Texture coordinates for fragment shader
    );
}

// =============================================================================
// FRAGMENT SHADER RESOURCES: What the fragment shader can access
// =============================================================================

/// The screen capture texture - contains the image data we want to display
/// @group(0) @binding(0) corresponds to binding 0 in our bind group layout
/// This texture contains the actual screen pixels (currently test pattern, later real screen capture)
@group(0) @binding(0)
var t_screen: texture_2d<f32>;

/// The sampler - controls how we read pixels from the texture
/// @group(0) @binding(1) corresponds to binding 1 in our bind group layout  
/// This determines how to interpolate/filter when reading texture pixels
@group(0) @binding(1)
var s_screen: sampler;

// =============================================================================
// FRAGMENT SHADER: Determines the color of each pixel
// =============================================================================

/// Fragment shader entry point
/// 
/// WHAT THIS DOES:
/// For every pixel on the screen, this function runs and decides what color it should be
/// We sample (read) from our screen capture texture and return that color
/// 
/// HOW FRAGMENT SHADERS WORK:
/// - Runs once for every pixel that needs to be drawn
/// - Receives pixel position as input (frag_coord)
/// - Must output final pixel color as return value
/// - Runs massively in parallel (thousands of pixels processed simultaneously)
@fragment  
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    // Sample color from the screen capture texture at the interpolated coordinates
    // This displays our uploaded texture data (currently test pattern, later real screen capture)
    return textureSample(t_screen, s_screen, input.tex_coords);
}

// =============================================================================
// SUMMARY OF SHADER PIPELINE:
// =============================================================================
//
// 1. VERTEX SHADER (vs_main):
//    - GPU calls this 3 times (once per vertex)
//    - Generates positions for 1 large triangle covering entire screen
//    - Output: 3 vertices positioned to cover entire screen (with clipping)
//
// 2. RASTERIZATION (automatic):
//    - GPU automatically fills in pixels between vertices
//    - Creates fragments (potential pixels) for every pixel inside triangles
//    - Each fragment gets passed to fragment shader
//
// 3. FRAGMENT SHADER (fs_main):
//    - GPU calls this once for every pixel on screen
//    - Converts pixel position to texture coordinates  
//    - Samples color from screen capture texture
//    - Output: Final color for that pixel
//
// 4. RESULT:
//    - Every pixel displays the corresponding pixel from screen capture texture
//    - Creates perfect 1:1 mirror of captured content
//    - Ready for overlaying black boxes later (in future iterations)