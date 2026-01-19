void main() {
    vec2 uv = gl_FragCoord.xy / iResolution.xy;
    fragColor = texture(sampler2D(iTexture, iSampler), uv);
}
