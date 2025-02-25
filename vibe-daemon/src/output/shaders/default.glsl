    const float GAMMA = 2.2;
    const vec3 WHITE = vec3(1.);

    vec2 uv = gl_FragCoord.xy / iResolution.xy;

    vec2 zuv = uv * float(freqs.length());
    int id = int(floor(zuv.x));
    float y = 1. - uv.y;

    // check if we are within a bar
    if (y <= freqs[id]) {
        vec3 bottom_color = sin(vec3(2., 4., 8.) + iTime) * .2 + .5;
        float presence = step(y, freqs[id]);
    
        vec3 col = mix(bottom_color, WHITE, y) * presence;
    
        // apply gamma correction
        col.x = pow(col.x, GAMMA);
        col.y = pow(col.y, GAMMA);
        col.z = pow(col.z, GAMMA);
    
        fragColor = vec4(col, y);
    }
