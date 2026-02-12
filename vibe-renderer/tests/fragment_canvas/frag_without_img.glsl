void main() {
    vec2 uv = (2. * gl_FragCoord.xy - iResolution.xy) / iResolution.y;
    float r = length(uv);
    
    fragColor = vec4(r, freqs[3], sin(iTime), 1.);
}
