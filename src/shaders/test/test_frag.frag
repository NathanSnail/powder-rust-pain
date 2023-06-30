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
}; // +4

struct Sprite {
	vec2 pos; // 8
	vec2 size; // 16
	vec2 offset; // 24
	vec2 scale; // 32
	bool deleted; // 36
}; // +4

layout(binding = 0) buffer Data {
	Material mat[];
}
buf;

layout(binding = 1) buffer Sprites {
	Sprite sprites[];
}
sprite_buf;

layout(set = 0, binding = 2) uniform sampler2D atlas;

layout( push_constant ) uniform PushType
{
	vec2 dims;
} PushConstants;

layout(location = 0) out vec4 f_color;

void main() {
	Sprite _1 = sprite_buf.sprites[0];
	float radius = 0.02/2.0*1.2; // coeff to hide bg
	vec2 uv = gl_FragCoord.xy / PushConstants.dims;
	vec3 c = vec3(0.4,0.45,1.0);
	for(int i = 0; i < buf.mat.length(); i++)
	{
		if (length(buf.mat[i].pos-uv) < radius)
		{
			c = buf.mat[i].colour;
			break; // + ~10% fps
		}
	}
	for (int i = 0; i < sprite_buf.sprites.length(); i++)
	{
		vec2 local = uv - sprite_buf.sprites[i].pos;
		if ((local.x < sprite_buf.sprites[i].size.x) && (local.y < sprite_buf.sprites[i].size.y) && (local.x > 0.0) && (local.y > 0.0) && !sprite_buf.sprites[i].deleted)  // bounding check
		{
			local = vec2(local.x * sprite_buf.sprites[i].scale.x, local.y * sprite_buf.sprites[i].scale.y); // cross product
			vec4 val = texture(atlas,local);
			c = val.a * val.rgb + c * (1.0 - val.a); // standard alpha recombination
			// c = val.rgb;
		}
	}

	// vec4 col = texture(atlas,uv * 2.0 - 1.0);
	f_color = vec4(c, 1.);
}