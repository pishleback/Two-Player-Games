struct VertexOut {
    @location(0) color: vec4<f32>,
    @location(1) tex_coords: vec2<f32>,
    @builtin(position) position: vec4<f32>,
};

struct Uniforms {
    rotation: mat4x4<f32>,
};

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

struct VertexIn {
    @location(0) position: vec2<f32>,
    @location(1) color: vec4<f32>,
    @location(2) tex_coords: vec2<f32>,
};

@group(0) @binding(1)
var t_diffuse: texture_2d<f32>;
@group(0) @binding(2)
var s_diffuse: sampler;

@vertex
fn vs_main(vertex: VertexIn) -> VertexOut {
    var out: VertexOut;
    out.position = vec4<f32>(
        vertex.position.x,
        vertex.position.y,
        0.0,
        1.0
    );
    out.color = vertex.color;
    out.tex_coords = vertex.tex_coords;
    return out;
}

@fragment
fn fs_main(in: VertexOut) -> @location(0) vec4<f32> {
    return textureSample(t_diffuse, s_diffuse, in.tex_coords);
}