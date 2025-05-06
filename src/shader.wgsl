struct Size {
    width: f32,
    height: f32
}

struct Time {
    s: f32
}

struct Camera {
    position: vec3<f32>,
    _pad: f32,
    direction: vec3<f32>,
    _pad2: f32,
};

struct Particle {
    position: vec3<f32>,
    radius: f32
}

@group(0) @binding(0)
var<uniform> u_screen: Size;

@group(1) @binding(0)
var<uniform> u_time: Time;

@group(2) @binding(0)
var<uniform> u_camera: Camera;

@group(3) @binding(0)
var<storage, read> u_particles: array<Particle>;

@group(3) @binding(1)
var<uniform> u_particles_len: u32;

const NUM_OF_STEPS = 128;
const MIN_DIST_TO_SDF = 0.001;
const MAX_DIST_TO_TRAVEL = 64.0;

fn smooth_min(d1: f32, d2: f32, k: f32) -> f32 {
    let h = clamp(0.5 + 0.5 * (d2 - d1) / k, 0.0, 1.0);
    return mix(d2, d1, h) - k * h * (1.0 - h);
}

fn sdf_sphere(p: vec3<f32>, c: vec3<f32>, r: f32) -> f32 {
    return length(p - c) - r;
}

fn sdf(o: vec3<f32>) -> f32 {
    var sm = MAX_DIST_TO_TRAVEL;
    for (var i: u32 = 0; i < u_particles_len; i++) {
        sm = smooth_min(
            sm,
            sdf_sphere(
                o,
                (sin(u_time.s)+1.0) * u_particles[i].position,
                u_particles[i].radius
            ),
            0.5);
    }
    return sm;
}

fn raymarch(orig: vec3<f32>, dir: vec3<f32>) -> f32 {
    var dist = 0.0;
    for (var i = 0; i < NUM_OF_STEPS; i++) {
        let p = orig + dist * dir;
        let d = sdf(p);
        if d < MIN_DIST_TO_SDF {
            break;
        }
        dist += d;
        if dist > MAX_DIST_TO_TRAVEL {
            break;
        }
    }
    return dist;
}

fn normal(p: vec3<f32>) -> vec3<f32> {
    let d = vec2(0.01, 0.0);
    let gx = sdf(p + d.xyy) - sdf(p - d.xyy);
    let gy = sdf(p + d.yxy) - sdf(p - d.yxy);
    let gz = sdf(p + d.yyx) - sdf(p - d.yyx);
    let normal = vec3(gx, gy, gz);
    return normalize(normal);
}

@vertex
fn vs_main(@builtin(vertex_index) index: u32) -> @builtin(position) vec4<f32> {
    var pos = array(
        vec2(-1.0, -1.0), vec2( 1.0, -1.0), vec2(-1.0,  1.0),
        vec2(-1.0,  1.0), vec2( 1.0, -1.0), vec2( 1.0,  1.0),
    );
    return vec4(pos[index], 0.0, 1.0);
}

@fragment
fn fs_main(@builtin(position) frag_coord: vec4<f32>) -> @location(0) vec4<f32> {
    let resolution = vec2(u_screen.width, u_screen.height); // pass as uniform if needed
    var uv = (frag_coord.xy / resolution) * 2.0 - vec2(1.0, 1.0);
    let aspect = resolution.x / resolution.y;

    uv = -uv;

    let forward = normalize(u_camera.direction); // already in your coordinate space
    let up = vec3(0.0, 1.0, 0.0); // Y-up
    let right = normalize(cross(forward, up)); // X right
    let camera_up = cross(right, forward);     // recomputed up (orthogonalized)

    let fov_scale = tan(radians(90.0) * 0.5); // or pass as uniform

    // Build ray direction using Y-up, Z-forward convention
    let ray_dir = normalize(
        forward +
        uv.x * aspect * fov_scale * right +
        uv.y * fov_scale * camera_up
    );

    let ray_origin = u_camera.position;

    var color = vec3<f32>();
    let dist = raymarch(ray_origin, ray_dir);

    if dist < MAX_DIST_TO_TRAVEL {
        color = vec3(1.0,1.0,1.0);
        let normal = normal(ray_origin + ray_dir * dist);
        let d = dot(normal, -u_camera.direction);
        color = vec3(d,d,d);
    } else {
        color = mix(vec3(0.58, 0.529, 0.459), vec3(0.714, 0.812, 0.78), saturate(((ray_dir.y/0.01) + 1.0)/2.0));
    }
    return vec4(color, 1.0);
}
