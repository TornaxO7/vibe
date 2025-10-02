@group(0) @binding(0)
var output: texture_storage_2d<r16unorm, write>;

struct Data {
    canvas_size: f32,
    pattern: u32,
}

@group(0) @binding(1)
var<uniform> data: Data;

const BOX: u32 = 0;
const CIRCLE: u32 = 1;
const HEART: u32 = 2;

// helper functions
fn dot2(v: vec2f) -> f32 {
    return dot(v, v);
}

// https://iquilezles.org/articles/distfunctions2d/
fn sdfCircle(uv: vec2f) -> f32
{
    const RADIUS: f32 = .4;
    return length(uv) - RADIUS;
}

fn sdfBox(uv: vec2f) -> f32
{
    let d = abs(uv) - vec2f(.4);
    return length(max(d, vec2f(0.0))) + min(max(d.x, d.y), 0.0);
}

fn sdfHeart(uv: vec2f) -> f32
{
    var p = uv * 1.25;
    p.y -= .5;
    p.y *= -1.;
    
    p.x = abs(p.x);

    if (p.y+p.x>1.0) {
        return sqrt(dot2(p-vec2f(0.25,0.75))) - sqrt(2.0)/4.0;
    }
    return sqrt(
                min(
                    dot2(p-vec2f(0.00,1.00)),
                    dot2(p-0.5*max(p.x+p.y,0.0))
                )
            ) * sign(p.x-p.y);
}

@compute
@workgroup_size(16, 16, 1)
fn main(@builtin(global_invocation_id) gid: vec3u) {
    var uv = (2. * vec2f(gid.xy) - vec2f(data.canvas_size)) / data.canvas_size;

    var d: f32 = 0.;
    switch data.pattern {
        case BOX, default: {
            d = sdfBox(uv);
        }
        case CIRCLE: {
            d = sdfCircle(uv);
        }
        case HEART: {
            d = sdfHeart(uv);
        }
    }

    let glow = min(1., .1 / max(1e-5, d) - .15);
    textureStore(output, gid.xy, vec4f(glow, 0., 0., 0.));
}
