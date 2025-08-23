@group(0) @binding(0)
var<uniform> iResolution: vec2<f32>;

@group(0) @binding(1)
var<uniform> pattern: u32;

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
    const RADIUS: f32 = .5;
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
@fragment
fn main(@builtin(position) pos: vec4<f32>) -> @location(0) f32 {
    var uv = (2. * pos.xy - iResolution.xy) / iResolution.y;

    var d: f32 = 0.;
    switch pattern {
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
    return glow;
}
