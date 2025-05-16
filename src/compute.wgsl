struct Camera {
    position: vec3<f32>,
    aspect: f32,
    right: vec3<f32>,
    fov: f32,
    up: vec3<f32>,
    fov_scale: f32,
    forward: vec3<f32>,
    proj: mat4x4<f32>,
    view: mat4x4<f32>,
    inv_proj: mat4x4<f32>,
    inv_view: mat4x4<f32>,
};

struct Particle {
    position: vec3<f32>,
    radius: f32
}

// Must be the same as the one in particle.rs
const BUNDLE_SIZE = 32;

@group(0) @binding(0)
var<storage> particles: array<Particle, BUNDLE_SIZE>;

@group(0) @binding(1)
var sdf_tex_write: texture_storage_3d<rgba16float, write>;

@group(0) @binding(2)
var sdf_tex_read: texture_3d<f32>;

@group(0) @binding(3)
var<uniform> u_camera: Camera;

fn smin(d1: f32, d2: f32) -> f32 {
    let k = 0.1;
    let h = clamp(0.5 + 0.5 * (d2 - d1) / k, 0.0, 1.0);
    return mix(d2, d1, h) - k * h * (1.0 - h);
}

fn sdf_particle(p: vec3<f32>, particle: Particle) -> f32 {
    return length(p - particle.position) - 0.1;
}

fn sdf(p: vec3<f32>) -> f32 {
    var curr = sdf_particle(p, particles[0]);
    for (var i = 1; i < BUNDLE_SIZE; i++) {
        curr = min(curr, sdf_particle(p, particles[i]));
    }
    return curr;
}

// Converts a position in normalized screen space to world space.
// In normalized screen space, (0,0,0) is (left,bottom,near)
// and (1,1,1) is (right,top,far).
fn screen_to_world(pos: vec3<f32>) -> vec3<f32> {

    // Converts normalized screen space to normalized device space
    let ndc = vec3(
        pos.x * 2.0 - 1.0,
        pos.y * 2.0 - 1.0,
        pos.z
    );
    
    // Reverts the projection matrix
    let view_pos_hom = u_camera.inv_proj * vec4(ndc, 1.0);
    let view_pos    = view_pos_hom.xyz / view_pos_hom.w;

    // Reverts the view matrix
    let world_pos_hom = u_camera.inv_view * vec4(view_pos, 1.0);
    let world_pos = world_pos_hom.xyz / world_pos_hom.w;

    return world_pos;
}

@compute @workgroup_size(8,4,4)
fn cs_clear(@builtin(global_invocation_id) id: vec3<u32>) {
    textureStore(
        sdf_tex_write,
        vec3<i32>(id),
        vec4<f32>(100000.0, 0.0, 0.0, 0.0)
    );
}

@compute @workgroup_size(8,4,4)
fn cs_sdf(@builtin(global_invocation_id) id: vec3<u32>) {
    var value = textureLoad(
        sdf_tex_read,
        vec3<i32>(id),
        0
    );
    let size = vec3<f32>(textureDimensions(sdf_tex_write));
    let coord = vec3<f32>(id) + vec3<f32>(0.5, 0.5, 0.5); // center of voxel
    let norm = coord / size;
    value.r = min(value.r, sdf(screen_to_world(norm)));
    textureStore(
        sdf_tex_write,
        vec3<i32>(id),
        value
    );
}

