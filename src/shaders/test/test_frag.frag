#version 460

// layout(location=0) in vec2 v_tex_coords;
layout(location=0)out vec4 f_color;

void main(){
	vec2 uv=gl_FragCoord.xy/vec2(1920.,1080.);
	f_color=vec4(uv.x,uv.y,0.,1.);
}