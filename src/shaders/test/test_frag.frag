#version 460

struct Material {
	vec3 colour; // 12
	uint id; // 16
	vec2 pos; // 24
	vec2 vel; // 32
	vec2 target; // 40
	float mass; // 44
	float force; // 48
	float stable; // 52
	uint tags; // 56
	uint gas; // 60
};

struct Sprite {
	vec2 pos; // 8
	vec2 offset; // 16
};

layout(binding = 0) buffer Data {
	Material mat[];
}
buf;

layout(binding = 0) buffer Sprites {
	Sprite sprite[];
}
sprite;

layout( push_constant ) uniform PushType
{
	vec2 dims;
} PushConstants;

layout(location = 0) out vec4 f_color;

void main() {
	float radius = 0.02/2.0*1.2; // coeff to hide bg
	vec2 uv = gl_FragCoord.xy / PushConstants.dims;
	Material _ = buf.mat[0]; // needs to be used or it deletes the buffer, if we use it later this isn't needed - just for testing.
	vec3 c = vec3(0.4,0.45,1.0);
	for(int i = 0; i < buf.mat.length(); i++)
	{
		if (length(buf.mat[i].pos-uv) < radius)
		{
			c = buf.mat[i].colour;
		}
	}
	f_color = vec4(c, 1.);
}