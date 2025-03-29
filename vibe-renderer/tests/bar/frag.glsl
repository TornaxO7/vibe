layout(set = 1, binding = 1) uniform float iTime;

layout(location = 0) out vec4 fragColor;

void main() {
    fragColor = vec4(iTime);
}
