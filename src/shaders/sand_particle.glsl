#version 450

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

layout(local_size_x = 128, local_size_y = 1, local_size_z = 1) in;

// layout(push_constant) uniform PushConstantType {
//     uint cur_buffer;
// } pc;

layout(binding = 0) buffer Materials {
	Material[2] mat[];
}
buf;

void main() {
	uint cur_buffer = 0;
	uint idx = gl_GlobalInvocationID.x;
	buf.mat[cur_buffer][idx].vel.y += 0.0005;
	buf.mat[cur_buffer][idx].colour = vec3(float(buf.mat[cur_buffer].length())/4.0);
	for(int i = 0; i < buf.mat.length(); i++)
	{
		vec2 dir = buf.mat[cur_buffer][idx].pos-buf.mat[cur_buffer][i].pos;
		float size = length(dir); 
		if (size < 0.02 && i != idx) // diameter
		{
			// buf.mat[cur_buffer][idx].colour = vec3(1.0);
			buf.mat[cur_buffer][idx].vel += (0.02-size)*dir*50.0;
			// buf.mat[cur_buffer][idx].pos += dir/4.0;
		}
	}
	buf.mat[cur_buffer][idx].pos += buf.mat[cur_buffer][idx].vel/100.0;
	// buf.mat[idx][cur_buffer].pos = vec2(float(idx)/64.0);
	buf.mat[cur_buffer][idx].pos.x = min(1.0,max(buf.mat[cur_buffer][idx].pos.x,0.0));
	// buf.mat[cur_buffer][idx].pos.x = mod(buf.mat[cur_buffer][idx].pos.x,1.0);
	buf.mat[cur_buffer][idx].pos.y = min(1.0,max(buf.mat[cur_buffer][idx].pos.y,0.0));
	if (buf.mat[cur_buffer][idx].pos.x <= 0.0005)
	{
		buf.mat[cur_buffer][idx].pos.x += 0.0006;
	}
	else if (buf.mat[cur_buffer][idx].pos.x >= 0.995)
	{
		buf.mat[cur_buffer][idx].pos.x -= 0.0006;
	}

	if (buf.mat[cur_buffer][idx].pos.y <= 0.005)
	{
		buf.mat[cur_buffer][idx].pos.y += 0.0006;
	}
	else if (buf.mat[cur_buffer][idx].pos.y >= 0.995)
	{
		buf.mat[cur_buffer][idx].pos.y -= 0.0006;
	}
	buf.mat[cur_buffer][idx].vel *= 0.999;
}