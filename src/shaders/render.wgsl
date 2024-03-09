struct Vertex {
    @location(0) position: vec2f,
};
 
struct VSOutput {
    @builtin(position) position: vec4f,
};
 
@vertex fn vs_main(vert: Vertex) -> VSOutput {
    var vsOut: VSOutput;
    vsOut.position = vec4f(vert.position, 0.0, 1.0);
    return vsOut;
}
 
@fragment fn fs_main(vsOut: VSOutput) -> @location(0) vec4f {
    return vec4f(1.0, 1.0, 0.0, 1.0); // yellow
}
