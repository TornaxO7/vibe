void main() {
    vec2 col = gl_FragCoord.xy / iResolution.xy * iTime + freqs[3];
    fragColor = vec4(col, 1., 1.);
}
