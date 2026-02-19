struct VertexOut {
    @location(0) world_pos: vec3<f32>,
    @location(1) colour: vec4<f32>,
    @location(2) tex_uv: vec2<f32>,
    @location(3) tex_idx: f32,
    @location(4) colour_to_tex: f32,
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
@group(0) @binding(3)
var icons_array: texture_2d_array<f32>;
@group(0) @binding(4)
var icons_sampler: sampler;

struct VertexIn {
    @location(0) position: vec3<f32>,
    @location(1) colour: vec4<f32>,
    @location(2) tex_uv: vec2<f32>,
    @location(3) tex_idx: f32,
    @location(4) colour_to_tex: f32,
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
    out.colour = vertex.colour;
    out.tex_uv = vertex.tex_uv;
    out.tex_idx = vertex.tex_idx;
    out.colour_to_tex = vertex.colour_to_tex;
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

    return in.colour_to_tex * textureSample(
        icons_array,
        icons_sampler,
        vec2<f32>(in.tex_uv.x, in.tex_uv.y),
        i32(round(in.tex_idx)),
    ) + (1 - in.colour_to_tex) * in.colour;
    
}