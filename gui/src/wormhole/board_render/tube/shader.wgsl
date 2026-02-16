struct VertexOut {
    @location(0) color: vec4<f32>,
    @location(1) world_pos: vec3<f32>,
    @builtin(position) position: vec4<f32>,
};

struct Uniforms {
    mat: mat4x4<f32>,
    side_length: f32,
    face_offset: f32,
    colours : array<vec4<f32>, 144>,
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
    out.position = uniforms.mat * vec4<f32>(
        vertex.position.x,
        vertex.position.y,
        vertex.position.z,
        1.0
    );
    out.world_pos = vertex.position;
    out.color = vertex.color;
    return out;
}

@fragment
fn fs_main(in: VertexOut) -> @location(0) vec4<f32> {
    var n = 0;

    var z = in.world_pos.z / uniforms.face_offset;

    let scale = uniforms.side_length / 8.0;
    // The world position projected onto the xy-plane relative to the squares on the board
    var sq_pos = vec2<f32>(in.world_pos.x / scale, in.world_pos.y / scale);
    // Discard outside the middle hole
    let dist2 = sq_pos.x * sq_pos.x + sq_pos.y * sq_pos.y;
    if (dist2 > 5.0) {
        discard;
    }

    var colour = false;
    if z < 0.0 {
        z = -z;
        colour = !colour;
        n += 1;
    }
    if z < 0.65 {
        z = -z;
        colour = !colour;
        n += 2;
    }
    var m = n + 18;
    if sq_pos.x < 0 {
        sq_pos.x = -sq_pos.x;
        colour = !colour;
        m += 8;
        n += 4;
    }
    if sq_pos.y < 0 {
        sq_pos.y = -sq_pos.y;
        colour = !colour;
        m += 16;
        n += 8;
    }
    if sq_pos.x > sq_pos.y {
        sq_pos = vec2<f32>(sq_pos.y, sq_pos.x);
        m += 64;
    }
    let normal = vec2<f32>(2.55, -1.0);
    if (dot(sq_pos - vec2<f32>(1.0, 2.0), normal) > 0) {
        colour = !colour;
        n += 128;
    } else {
        n = m;
    }

    return uniforms.colours[n];
}