#version 450

layout(binding = 0) buffer Data {
	float dims[];
}buf;

layout(location = 0) out vec4 f_color;

void main() {
vec2 uv = gl_FragCoord.xy / vec2(dims[0], dims[1]);
f_color = vec4(uv.x, uv.y, 0., 1.);
}