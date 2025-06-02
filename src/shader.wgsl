struct Screen {
    size: vec2<f32>,
}

struct Time {
    s: f32,
}

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

struct Intersection {
    is_hit: bool,
    hit_dist: f32,
}

@group(0) @binding(0)
var<uniform> u_screen: Screen;

@group(0) @binding(1)
var<uniform> u_camera: Camera;

@group(0) @binding(2)
var<uniform> u_time: Time;

@group(0) @binding(3)
var sdf_tex_read: texture_3d<f32>;

// @group(3) @binding(0)
// var<storage, read> u_particles: array<Particle>;

// @group(3) @binding(1)
// var<uniform> u_particles_len: u32;

const NUM_OF_STEPS = 128;
const MIN_DIST_TO_SDF = 0.005;
const MAX_DIST_TO_TRAVEL = 64.0;
const SUN_DIR = vec3(-1.0,-1.0,1.0) / 1.73205080757;

fn sdf_box(p: vec3<f32>, size: vec3<f32>) -> f32 {
    let q = abs(p-size/2) - size/2;
    return length(max(q,vec3(0.0,0.0,0.0))) + min(max(q.x,max(q.y,q.z)),0.0);
}

fn world_to_ubox(world_pos: vec3<f32>) -> vec3<f32> {
    return world_pos;
}

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

fn sample_linear_3d(
    tex: texture_3d<f32>,
    uvw: vec3<f32>,
    tex_size: vec3<u32>
) -> f32 {
    let size_f = vec3<f32>(tex_size);
    let pos = uvw * size_f - vec3<f32>(0.5); // Shift to texel space
    let base = vec3<i32>(floor(pos));        // Integer base coordinate
    let frac = fract(pos);                   // Interpolation weights

    // Load 8 surrounding voxels
    let c000 = textureLoad(tex, clamp(base + vec3(0, 0, 0), vec3<i32>(0,0,0), vec3<i32>(tex_size)), 0).r;
    let c100 = textureLoad(tex, clamp(base + vec3(1, 0, 0), vec3<i32>(0,0,0), vec3<i32>(tex_size)), 0).r;
    let c010 = textureLoad(tex, clamp(base + vec3(0, 1, 0), vec3<i32>(0,0,0), vec3<i32>(tex_size)), 0).r;
    let c110 = textureLoad(tex, clamp(base + vec3(1, 1, 0), vec3<i32>(0,0,0), vec3<i32>(tex_size)), 0).r;
    let c001 = textureLoad(tex, clamp(base + vec3(0, 0, 1), vec3<i32>(0,0,0), vec3<i32>(tex_size)), 0).r;
    let c101 = textureLoad(tex, clamp(base + vec3(1, 0, 1), vec3<i32>(0,0,0), vec3<i32>(tex_size)), 0).r;
    let c011 = textureLoad(tex, clamp(base + vec3(0, 1, 1), vec3<i32>(0,0,0), vec3<i32>(tex_size)), 0).r;
    let c111 = textureLoad(tex, clamp(base + vec3(1, 1, 1), vec3<i32>(0,0,0), vec3<i32>(tex_size)), 0).r;

    // Interpolate
    let c00 = mix(c000, c100, frac.x);
    let c10 = mix(c010, c110, frac.x);
    let c01 = mix(c001, c101, frac.x);
    let c11 = mix(c011, c111, frac.x);

    let c0 = mix(c00, c10, frac.y);
    let c1 = mix(c01, c11, frac.y);

    let c = mix(c0, c1, frac.z);

    return c;
}


fn sdf(p: vec3<f32>) -> f32 {
    let norm = world_to_ubox(p);
    return sample_linear_3d(sdf_tex_read, norm, textureDimensions(sdf_tex_read));
}

//https://tavianator.com/2015/ray_box_nan.html
fn box_intersection(box_min: vec3<f32>, box_max: vec3<f32>, orig: vec3<f32>, dir: vec3<f32>) -> Intersection {
    var t1 = (box_min.x - orig.x)/dir.x;
    var t2 = (box_max.x - orig.x)/dir.x;

    var tmin = min(t1, t2);
    var tmax = max(t1, t2);

    t1 = (box_min.y - orig.y)/dir.y;
    t2 = (box_max.y - orig.y)/dir.y;

    tmin = max(tmin, min(min(t1, t2), tmax));
    tmax = min(tmax, max(max(t1, t2), tmin));

    t1 = (box_min.z - orig.z)/dir.z;
    t2 = (box_max.z - orig.z)/dir.z;

    tmin = max(tmin, min(min(t1, t2), tmax));
    tmax = min(tmax, max(max(t1, t2), tmin));

    tmin = max(tmin, 0.0);

    return Intersection(tmax > tmin, tmin);
}


fn raymarch(orig: vec3<f32>, dir: vec3<f32>) -> f32 {
    let inter = box_intersection(vec3(0.0,0.0,0.0),vec3(1.0,1.0,1.0),orig,dir);
    if !inter.is_hit {
        return MAX_DIST_TO_TRAVEL;
    }
    var dist = inter.hit_dist;
    var p = orig + dist * dir;
    for (var i = 0; i < NUM_OF_STEPS; i++) {
        let d = sdf(p);
        if d < MIN_DIST_TO_SDF {
            break;
        }
        dist += d;
        p = orig + dist * dir;
        if p.x < 0 || p.y < 0 || p.z < 0 || p.x > 1 || p.y > 1 || p.z > 1 {
            return MAX_DIST_TO_TRAVEL;
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

fn sky_color(n: vec3<f32>) -> vec3<f32> {
    return mix(
        vec3(0.58, 0.529, 0.459),
        vec3(0.714, 0.812, 0.78),
        saturate(((n.y/0.01) + 1.0)/2.0)
    );
}

fn sky_color_diffuse(n: vec3<f32>) -> vec3<f32> {
    return mix(
        vec3(0.58, 0.529, 0.459),
        vec3(0.714, 0.812, 0.78),
        saturate(((n.y/0.5) + 1.0)/2.0)
    );
}
@vertex
fn vs_main(@builtin(vertex_index) index: u32) -> @builtin(position) vec4<f32> {
    var pos = array(
        vec2(-1.0, -1.0), vec2( 1.0, -1.0), vec2(-1.0,  1.0),
        vec2(-1.0,  1.0), vec2( 1.0, -1.0), vec2( 1.0,  1.0),
    );
    return vec4(pos[index], 0.0, 1);
}

fn screen_to_uv(screen_pos: vec2<f32>) -> vec2<f32> {
    let normalized_pos_yn = screen_pos / u_screen.size;
    let normalized_pos_yp = vec2(normalized_pos_yn.x, 1-normalized_pos_yn.y);
    return normalized_pos_yp;
}

fn screen_to_ndc(screen_pos: vec2<f32>) -> vec2<f32> {
    let normalized_pos_yn = screen_pos / u_screen.size;
    let normalized_pos_yp = vec2(normalized_pos_yn.x, 1-normalized_pos_yn.y);
    let ndc = normalized_pos_yp * 2 - vec2(1,1);
    return ndc;
}

@fragment
fn fs_main(@builtin(position) screen_pos: vec4<f32>) -> @location(0) vec4<f32> {

    var uv = screen_to_uv(screen_pos.xy);
    var near = screen_to_world(vec3(uv,0));
    var far = screen_to_world(vec3(uv,1));

    let ray_origin = u_camera.position;
    let ray_dir = normalize(far - near);

    let dist = raymarch(ray_origin, ray_dir);

    var color = vec3<f32>();

    if dist < MAX_DIST_TO_TRAVEL {
        color = vec3(1.0,1.0,1.0);
        let normal = normal(ray_origin + ray_dir * dist);
        let phong = saturate(dot(normal, -SUN_DIR));
        let ambient = sky_color_diffuse(normal);
        color = vec3(phong,phong,phong) + 0.2 * ambient;
    } else {
        color = sky_color(ray_dir);
    }
    return vec4(color, 1.0);
}
