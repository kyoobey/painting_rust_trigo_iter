


struct VertexInput {
	[[location(0)]] position: vec2<f32>;
};

struct VertexOutput {
	[[builtin(position)]] clip_position: vec4<f32>;
	[[location(0)]] uv: vec2<f32>;
};

struct Uniforms {
	time: f32;
};

[[group(0), binding(0)]]
var<uniform> uniforms: Uniforms;



[[stage(vertex)]]
fn vertex_shader_main(
	model: VertexInput
) -> VertexOutput {
	var out: VertexOutput;

	out.clip_position = vec4<f32>(model.position.x, model.position.y, 0., 1.);
	out.uv = model.position;

	return out;
}


// var<private> AA: u32 = 1;
var<private> PI: f32 = 3.14159265359;
var<private> TAU: f32 = 6.28318530718;


fn iterate(p: vec2<f32>, t: vec4<f32>) -> vec2<f32> {

	return p - 0.05 * cos(t.xz + p.x*p.y + cos(t.yw + 1.5 * PI * p.yx) + p.yx * p.yx );
}



[[stage(fragment)]]
fn fragment_shader_main(
	input: VertexOutput
) -> [[location(0)]] vec4<f32> {
	var uv = input.uv * 3.0;

	var time = uniforms.time * TAU/ 60.0;
	var t = time * vec4<f32>(1., -1., 1., -1.) + vec4<f32>(0., 2., 3., 1.);

	var tot = vec3<f32>(0.);


	for (var m=0; m < 2; m = m + 1) {
		for (var n=0; n < 2; n = n + 1) {
			var z = uv;
			var s = vec3<f32>(0.);
			for (var i=0; i < 100; i = i + 1) {
				z = iterate(z, t);
				var d = dot(z-uv, z-uv);
				s.x = s.x + 1.0 / (0.1 + d);
				s.y = s.y + sin(atan2(uv.x - z.x, uv.y - z.y));
				s.z = s.z + exp(-0.2 * d);
			}
			s = s * 0.0025;
			var col = 0.5 + 0.5 * cos(vec3<f32>(2.5 + s.z * TAU));
			tot = tot + col;
		}
	}

	return vec4<f32>(tot, 1.);

}


