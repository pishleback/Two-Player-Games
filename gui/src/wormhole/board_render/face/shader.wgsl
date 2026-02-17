struct VertexOut {
    @location(0) world_pos: vec3<f32>,
    @builtin(position) position: vec4<f32>,
};

struct Uniforms {
    pixels_size: vec2<u32>,
    mat: mat4x4<f32>,
    side_length: f32,
    face_offset: f32,
    colours : array<vec4<f32>, 144>,
};

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;
@group(0) @binding(1)
var depth_peel_view: texture_depth_2d;
@group(0) @binding(2)
var depth_peel_sample: sampler;

struct VertexIn {
    @location(0) position: vec3<f32>,
};

@vertex
fn vs_main(vertex: VertexIn) -> VertexOut {
    var out: VertexOut;
    out.position = uniforms.mat * vec4<f32>(
        vertex.position.x,
        vertex.position.y,
        vertex.position.z,
        1.0
    );
    out.world_pos = vertex.position;
    return out;
}

@fragment
fn fs_main(in: VertexOut) -> @location(0) vec4<f32> {
    // Discard if at or closer than the depth peel texture
    let depth_peel_value = textureSample(
        depth_peel_view, depth_peel_sample, 
        vec2<f32>(in.position.x / f32(uniforms.pixels_size.x), in.position.y / f32(uniforms.pixels_size.y))
    );
    if in.position.z <= depth_peel_value {
        discard;
    }

    let scale = uniforms.side_length / 8.0;
    // The world position projected onto the xy-plane relative to the squares on the board
    let sq_pos = vec2<f32>(in.world_pos.x / scale, in.world_pos.y / scale);

    // Discard for the hole in the middle
    let dist2 = sq_pos.x * sq_pos.x + sq_pos.y * sq_pos.y;
    if (dist2 < 5.0) {
        discard;
    }

    // Add checkerboard pattern
    let isq_pos = vec2<i32>(floor(sq_pos));
    var checker = (isq_pos.x + isq_pos.y) & 1;
    var n = clamp(isq_pos.x + 4, 0, 7) + 8 * clamp(isq_pos.y + 4, 0, 7);
    if (in.world_pos.z < 0.0) {
        checker ^= 1; // Invert the pattern of white / black on the reverse side
        n += 64;
    }

    // Special cases for the 8 pentagons where part of the square is visible but its number isn't the expected grid number
    if n == 18 {
        n = 140;
    } else if n == 21 {
        n = 136;
    } else if n == 42 {
        n = 132;
    } else if n == 45 {
        n = 128;
    } else if n == 82 {
        n = 141;
    } else if n == 85 {
        n = 137;
    } else if n == 106 {
        n = 133;
    } else if n == 109 {
        n = 129;
    }

    return uniforms.colours[n];
}