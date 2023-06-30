#version 450

// these numbers are memory byte allignment information, not really relevant unless you change the structs at all.

struct Material {
	vec3 colour; // 12
	uint id; // 16 (unused) for binding forces
	vec2 pos; // 24
	vec2 vel; // 32
	vec2 target; // 40 attractor point
	float mass; // 44
	float force; // 48 amount of attraction to target
	float stable; // 52 amount of resistance to pushing before target decouples
	uint tags; // 56 (unused) bit flags
	uint gas; // 60 antigrav
}; // +4

struct Hitbox {
	vec2 pos; // 8 (hitbox owns the real entity position so that buffers are better)
	vec2 size; // 16
	vec2 vel;
	float mass; // 20
	bool simulate; // 24 (booleans are not glbooleans and so are 32 bit alligned meaning 4 byte memory blocks)
}; // +0

layout(local_size_x = 64, local_size_y = 1, local_size_z = 1) in;

layout(binding = 0) buffer DataMaterial { // eventually need double buffer for non random results
	Material mat[];
}
buf;

layout(binding = 1) buffer DataEntity { // eventually need double buffer for non random results
	Hitbox ent[];
}
entity;

float random (vec2 st) {
    return fract(sin(dot(st.xy,vec2(12.9898,78.233)))*43758.5453123)*2.0-1.0; // https://thebookofshaders.com/10/
}

void main() {
	uint idx = gl_GlobalInvocationID.x;
	float radius = 0.02;
	buf.mat[idx].vel.y += 0.0005;
	for(int i = 0; i < buf.mat.length(); i++)
	{
		vec2 dir = buf.mat[idx].pos-buf.mat[i].pos;
		float size = length(dir); 
		if (size < radius && i != idx) // diameter
		{
			buf.mat[idx].vel += pow((radius-size)*(1.0/radius),0.5)*dir;
			// buf.mat[idx].pos += dir/4.0;
		}
	}

	for(int i = 0; i < entity.ent.length(); i++)
	{
		vec2 local = buf.mat[idx].pos - entity.ent[i].pos;
		if ((local.x < entity.ent[i].size.x) && (local.y < entity.ent[i].size.y) && (local.x > 0.0) && (local.y > 0.0))  // bounding check
		{
			local -= entity.ent[i].size / 2.0;
			float mag = length(entity.ent[i].size / 2.0) - abs(length(local));
			entity.ent[i].vel -= normalize(local) * mag / entity.ent[i].mass;
			buf.mat[idx].vel += normalize(local) * mag / buf.mat[idx].mass;
		}
	}

	buf.mat[idx].vel += vec2(random(buf.mat[idx].vel+buf.mat[idx].pos),random(buf.mat[idx].pos*2.0-buf.mat[idx].vel))/10000.0; // helps edges
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
	float max_speed = 0.1;
	if(length(buf.mat[idx].vel)>max_speed)
	{
		buf.mat[idx].vel = buf.mat[idx].vel / length(buf.mat[idx].vel) * max_speed;
	}
	buf.mat[idx].vel *= 0.999;
}