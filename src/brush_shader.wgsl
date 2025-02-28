struct VertexInput {
    @builtin(vertex_index) vertex_index : u32,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    // @location(0) tex_coords: vec2<f32>
};

@vertex
fn vs_main(
    @builtin(vertex_index) vertex_index : u32,
) -> VertexOutput {
    // float x = float(((uint(gl_VertexID) + 2u) / 3u)%2u); 
    // float y = float(((uint(gl_VertexID) + 1u) / 3u)%2u); 

    // gl_Position = vec4(-1.0f + x*2.0f, -1.0f+y*2.0f, 0.0f, 1.0f);

    let x: f32 = f32(((vertex_index + 2u) /  3u) % 2u);
    let y: f32 = f32(((vertex_index + 1u) /  3u) % 2u);

    var out: VertexOutput;
    out.clip_position = vec4<f32>(-1.0 + x * 2.0, -1.0 + y * 2.0, 0.0, 1.0);
    return out;
}

struct BrushUniform {
    position: vec3<f32>,
    radius: f32
};
@group(0) @binding(0)
var<uniform> brushUniform: BrushUniform;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let frag_pos = in.clip_position.xy;
    let brush_pos = brushUniform.position.xy;
    let distance = length(frag_pos - brush_pos);
    if (distance <= brushUniform.radius) {
        return vec4<f32>(0.0, 1.0, 0.0, 1.0);
    }
    return vec4<f32>(0.0);
}

