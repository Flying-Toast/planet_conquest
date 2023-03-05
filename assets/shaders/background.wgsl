#import bevy_sprite::mesh2d_types
#import bevy_sprite::mesh2d_view_bindings

struct BackgroundMaterial {
	base_color: vec4<f32>,
	noise_color: vec4<f32>,
}

@group(1) @binding(0)
var<uniform> material: BackgroundMaterial;

fn rand(seed: vec3<f32>) -> f32 {
	return fract(sin(dot(seed, vec3(12.9898, 78.233, 195.3876))) * 43758.5453);
}

fn rand_unit(seed: vec3<f32>) -> vec3<f32> {
	let theta = rand(seed) * 2. * 3.14159;
	let theta2 = rand(seed.yzx) * 2. * 3.14159;
	return vec3(cos(theta), sin(theta), sin(theta2));
}

fn interpolate(a: f32, b: f32, x: f32) -> f32 {
	return a + (6. * pow(x, 5.) - 15. * pow(x, 4.) + 10. * pow(x, 3.)) * (b - a);
}

fn noise(coord: vec3<f32>) -> f32 {
	let cell = floor(coord);
	let cell_inner = coord - cell;
	let corners = array(
		cell, //tl
		cell + vec3(1., 0., 0.), //tr
		cell + vec3(0., 1., 0.), //bl
		cell + vec3(1., 1., 0.), //br
		cell + vec3(0., 0., 1.), //tl
		cell + vec3(1., 0., 1.), //tr
		cell + vec3(0., 1., 1.), //bl
		cell + vec3(1., 1., 1.), //br
	);

	let gradients = array(
		rand_unit(corners[0]),
		rand_unit(corners[1]),
		rand_unit(corners[2]),
		rand_unit(corners[3]),
		rand_unit(corners[4]),
		rand_unit(corners[5]),
		rand_unit(corners[6]),
		rand_unit(corners[7]),
	);

	let offsets = array(
		corners[0] - coord,
		corners[1] - coord,
		corners[2] - coord,
		corners[3] - coord,
		corners[4] - coord,
		corners[5] - coord,
		corners[6] - coord,
		corners[7] - coord,
	);

	let dots = array(
		dot(offsets[0], gradients[0]),
		dot(offsets[1], gradients[1]),
		dot(offsets[2], gradients[2]),
		dot(offsets[3], gradients[3]),
		dot(offsets[4], gradients[4]),
		dot(offsets[5], gradients[5]),
		dot(offsets[6], gradients[6]),
		dot(offsets[7], gradients[7]),
	);

	let t1 = interpolate(dots[0], dots[1], cell_inner.x);
	let b1 = interpolate(dots[2], dots[3], cell_inner.x);
	let x1 = interpolate(t1, b1, cell_inner.y);

	let t2 = interpolate(dots[4], dots[5], cell_inner.x);
	let b2 = interpolate(dots[6], dots[7], cell_inner.x);
	let x2 = interpolate(t2, b2, cell_inner.y);

	return interpolate(x1, x2, cell_inner.z);
}

@fragment
fn fragment(
	#import bevy_sprite::mesh2d_vertex_output
) -> @location(0) vec4<f32> {
	let noise = noise(vec3(uv * 10., globals.time * 0.3));
	let c = vec3(material.noise_color.rgb * noise) + material.base_color.rgb;
	return vec4(c, 1.);
}
