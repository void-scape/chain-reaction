// https://godotshaders.com/shader/animated-diamond-pattern/

#import bevy_sprite::mesh2d_vertex_output::VertexOutput
#import bevy_render::globals::Globals

@group(0) @binding(1) var<uniform> globals: Globals;

// ############### GRID

// fn grid(uv_in: vec2<f32>, velocity: f32, iTime: f32) -> f32 {
//     var uv = uv_in;
//     let size = vec2<f32>(uv.y, uv.y * uv.y * 0.2) * 0.01;
//     uv += vec2<f32>(0.0, iTime * 4.0 * (velocity + 0.05));
//     uv = abs(fract(uv) - 0.5);
//     let lines = smoothstep(size, vec2<f32>(0.0), uv);
//     let lines_with_velocity = lines + smoothstep(size * 5.0, vec2<f32>(0.0), uv) * 0.4 * velocity;
//     return clamp(lines_with_velocity.x + lines_with_velocity.y, 0.0, 3.0);
// }
// 
// @fragment
// fn fragment(input: VertexOutput) -> @location(0) vec4<f32> {
//     // var uv = (2.0 * frag_coord.xy - screen_resolution) / screen_resolution.y;
//     let velocity = 0.30;
//     var uv = input.uv - vec2f(0.5, 0.5);
// 
//     var col = vec3<f32>(0.6, 0.1, 0.6);
// 
//     if (uv.y < 1.0) {
//         uv.y = 2.0 / (abs(uv.y) * 1.0);
//         uv.x *= uv.y - 1.01;
//         let gridVal = grid(uv, velocity, globals.time);
//         col = mix(col, vec3<f32>(0.6, 0.2, 0.10), gridVal);
//     }
// 
//     let mix_factor = sin(globals.time * 0.71) * 1.1;
//     col = mix(vec3<f32>(col.r, col.r, col.r) * 0.53, col, mix_factor);
// 
//     return vec4<f32>(col, 7.0);
// }

// ############### DIAMONDS

@fragment
fn fragment(input: VertexOutput) -> @location(0) vec4<f32> {
    var value: f32;
    let screen_pixel_size = 1.0 / vec2f(1024, 1024);
    let aspect = screen_pixel_size.y / screen_pixel_size.x;

    let rot = radians(45.0);
    let s = sin(rot);
    let c = cos(rot);

    var uv = input.uv - vec2f(0.5, 0.5);

    let newX = uv.x * c - uv.y * s;
    let newY = uv.x * s + uv.y * c;

    uv = vec2<f32>(newX, newY);

    uv += vec2<f32>(0.5, 0.5 * aspect);
    uv.y += 0.5 * (1.0 - aspect);

    let pos = 10.0 * uv;
    let rep = fract(pos);
    let dist = 2.0 * min(min(rep.x, 1.0 - rep.x), min(rep.y, 1.0 - rep.y));
    let squareDist = length((floor(pos) + vec2<f32>(0.5)) - vec2<f32>(5.0));

    var edge = sin(globals.time - squareDist * 0.5) * 0.5 + 0.5;
    edge = (globals.time - squareDist * 0.5) * 0.5;
    edge = 2.0 * fract(edge * 0.5);

    value = fract(dist * 2.0);
    value = mix(value, 1.0 - value, step(1.0, edge));
    edge = pow(abs(1.0 - edge), 2.0);
    value = smoothstep(edge - 0.05, edge, 0.95 * value);
    value += squareDist * 0.1;

    var color = mix(vec4<f32>(0.259, 0.141, 0.2, 1.0), vec4<f32>(0.141, 0.133, 0.204, 1.0), value);
    color.a = 0.25 * clamp(value, 0.0, 1.0);

    return color;
}



































































