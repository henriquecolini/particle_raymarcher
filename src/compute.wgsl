struct Particle {
    position: vec3<f32>,
    radius: f32
}

@group(0) @binding(0)
var<storage> particles: array<Particle, 16>;

@group(0) @binding(1)
var sdf_tex_write: texture_storage_3d<rgba16float, write>;

@group(0) @binding(2)
var sdf_tex_read: texture_3d<f32>;

fn smin(d1: f32, d2: f32) -> f32 {
    let k = 0.1;
    let h = clamp(0.5 + 0.5 * (d2 - d1) / k, 0.0, 1.0);
    return mix(d2, d1, h) - k * h * (1.0 - h);
}

fn sdf_particle(p: vec3<f32>, particle: Particle) -> f32 {
    return length(p - particle.position) - particle.radius;
}

fn sdf(p: vec3<f32>) -> f32 {
    return min(
        min(
            min(
                min(
                    sdf_particle(p, particles[0]),
                    sdf_particle(p, particles[1])
                ),
                min(
                    sdf_particle(p, particles[2]),
                    sdf_particle(p, particles[3])
                )
            ),
            min(
                min(
                    sdf_particle(p, particles[4]),
                    sdf_particle(p, particles[5])
                ),
                min(
                    sdf_particle(p, particles[6]),
                    sdf_particle(p, particles[7])
                )
            )
        ),
        min(
            min(
                min(
                    sdf_particle(p, particles[8]),
                    sdf_particle(p, particles[9])
                ),
                min(
                    sdf_particle(p, particles[10]),
                    sdf_particle(p, particles[11])
                )
            ),
            min(
                min(
                    sdf_particle(p, particles[12]),
                    sdf_particle(p, particles[13])
                ),
                min(
                    sdf_particle(p, particles[14]),
                    sdf_particle(p, particles[15])
                )
            )   
        )
    );
}

@compute @workgroup_size(8,4,2)
fn cs_clear(@builtin(global_invocation_id) id: vec3<u32>) {
    textureStore(
        sdf_tex_write,
        vec3<i32>(id),
        vec4<f32>(100000.0, 0.0, 0.0, 0.0)
    );
}

@compute @workgroup_size(8,4,2)
fn cs_sdf(@builtin(global_invocation_id) id: vec3<u32>) {
    var value = textureLoad(
        sdf_tex_read,
        vec3<i32>(id),
        0
    );
    let size = vec3<f32>(textureDimensions(sdf_tex_write));
    let coord = vec3<f32>(id) + vec3<f32>(0.5, 0.5, 0.5); // center of voxel
    let norm = coord / size;
    value.r = min(value.r, sdf(norm));
    textureStore(
        sdf_tex_write,
        vec3<i32>(id),
        value
    );
}

