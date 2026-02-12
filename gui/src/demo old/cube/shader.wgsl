struct VertexOut {
    @location(0) color: vec4<f32>,
    @builtin(position) position: vec4<f32>,
};

struct Uniforms {
    rotation: mat4x4<f32>,
};

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

struct VertexIn {
    @location(0) position: vec3<f32>,
    @location(1) color: vec4<f32>,
};

@vertex
fn vs_main(vertex: VertexIn) -> VertexOut {
    var out: VertexOut;
    out.position = uniforms.rotation * vec4<f32>(
        vertex.position.x,
        vertex.position.y,
        vertex.position.z,
        1.0
    );
    out.color = vertex.color;
    return out;
}

@fragment
fn fs_main(in: VertexOut) -> @location(0) vec4<f32> {
    return in.color;
}