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

layout(local_size_x = 64, local_size_y = 1, local_size_z = 1) in;

layout(binding = 0) buffer Data { // eventually need double buffer for non random results
	Material mat[];
}
buf;

void main() {
	uint idx = gl_GlobalInvocationID.x;
	buf.mat[idx].vel.y += 0.0005;
	for(int i = 0; i < buf.mat.length(); i++)
	{
		vec2 dir = buf.mat[idx].pos-buf.mat[i].pos;
		float size = length(dir); 
		if (size < 0.02 && i != idx) // diameter
		{
			buf.mat[idx].vel += pow((0.02-size)*50.0,0.5)*dir;
			// buf.mat[idx].pos += dir/4.0;
		}
	}
	buf.mat[idx].pos += buf.mat[idx].vel/100.0;
	buf.mat[idx].pos.x = min(1.0,max(buf.mat[idx].pos.x,0.0));
	// buf.mat[idx].pos.x = mod(buf.mat[idx].pos.x,1.0);
	// buf.mat[idx].pos = vec2(idx/64.0);
	buf.mat[idx].pos.y = min(1.0,max(buf.mat[idx].pos.y,0.0));
	if (buf.mat[idx].pos.x <= 0.0005)
	{
		buf.mat[idx].pos.x += 0.0006;
	}
	else if (buf.mat[idx].pos.x >= 0.995)
	{
		buf.mat[idx].pos.x -= 0.0006;
	}

	if (buf.mat[idx].pos.y <= 0.005)
	{
		buf.mat[idx].pos.y += 0.0006;
	}
	else if (buf.mat[idx].pos.y >= 0.995)
	{
		buf.mat[idx].pos.y -= 0.0006;
	}
	buf.mat[idx].vel *= 0.999;
}