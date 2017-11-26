extern crate cgmath;
extern crate image as im;
extern crate piston_window;

use cgmath::{ InnerSpace, Point3, Vector3 };
use im::{ RgbaImage };
use piston_window::*;

struct Sphere {
    center: Point3<f32>,
    radius: f32
}

impl Sphere {
    fn intersects(&self, ray: &Ray) -> Option<f32> {
        let radius_squared = self.radius * self.radius;
        let l = self.center - ray.origin;
        let tca = l.dot(ray.direction);

        if tca < 0.0 {
            return None;
        }

        let d2 = l.dot(l) - tca * tca;
        if d2 > radius_squared {
            return None;
        }

        let thc = (radius_squared - d2).sqrt();
        let t0 = tca - thc;
        let t1 = tca + thc;

        if t0 < 0.0 && t1 < 0.0 {
            return None;
        }

        // Return shortest distance along line
        return if t0 < t1 { Some(t0) } else { Some(t1) };
    }
}

struct Ray {
    origin: Point3<f32>,
    direction: Vector3<f32>
}

struct Camera {
    position: Point3<f32>,
    up: Vector3<f32>,
    at: Vector3<f32>,
    fov: f32
}

struct RenderOptions {
    width: u32,
    height: u32
}

struct Scene {
    spheres: Vec<Sphere>
}

fn closest_intersection(scene: &Scene, ray: Ray) -> Option<(&Sphere, f32)> {
    scene.spheres
         .as_slice()
         .into_iter()
         .filter_map(|s| { 
             let intersection = s.intersects(&ray);
             match intersection {
                 Some(i) => {
                     return if i.is_nan() { None } else { Some((s, i)) };
                 },
                 None => None
             }
         })
         .min_by(|x, y| { 
             let &(s1, i1) = x;
             let &(s2, i2) = y;
             return i1.partial_cmp(&i2).unwrap(); // Shouldn't ever hit NaN due to check above
         })
}

fn render_frame(scene: &Scene, camera: &Camera, render_options: &RenderOptions) -> RgbaImage {
    let mut img = RgbaImage::new(render_options.width, render_options.height);

    let aspect_ratio = (render_options.width as f32) / (render_options.height as f32);
    let theta = camera.fov.to_radians() / 2.0;
    let fov_scalar = theta.tan();

    for px_x in 0..render_options.width {
        for px_y in 0..render_options.height {
            // Calculate pixel NDC (normalized device coordinates)
            let px_ndc_x = ((px_x as f32) + 0.5) / (render_options.width as f32);
            let px_ndc_y = ((px_y as f32) + 0.5) / (render_options.height as f32);

            // Calculate pixel screen space coordinates
            let mut px_screen_x = 2.0 * px_ndc_x - 1.0;
            let mut px_screen_y = 1.0 - (2.0 * px_ndc_y);

            // Account for aspect ratio
            px_screen_x = px_screen_x * aspect_ratio;

            // Account for camera FoV (Field of View)
            px_screen_x = px_screen_x * fov_scalar;
            px_screen_y = px_screen_y * fov_scalar;

            let px_camera_space = Point3::new(px_screen_x, px_screen_y, -1.0);

            let ray_vector = (px_camera_space - camera.position).normalize();
            let ray = Ray { origin: camera.position, direction: ray_vector };

            let closest_intersection = closest_intersection(&scene, ray);
            match closest_intersection {
                Some(i) => {
                    let p = im::Rgba([0, 0, 0, 255]);
                    img.put_pixel(px_x, px_y, p);
                },
                None => {}
            }
        }
    }

    img
}

fn main() {
    let mut spheres = Vec::new();
    spheres.push(Sphere { center: Point3 { x: 2.0, y: 0.0, z: -5.0 }, radius: 1.0 });
    spheres.push(Sphere { center: Point3 { x: 2.0, y: 2.0, z: -4.0 }, radius: 0.9 });

    let mut scene = Scene { spheres: spheres };

    let camera = Camera {
        position: Point3 { x: 0.0, y: 0.0, z: 0.0 },
        up: Vector3  { x: 0.0, y: 1.0, z: 0.0 },
        at: Vector3 { x: 1.0, y: 0.0, z: 0.0 },
        fov: 90.0
    };
    let render_options = RenderOptions { width: 640, height: 640 };

    let opengl = OpenGL::V3_2;
    let mut window: PistonWindow =
        WindowSettings::new("rs-tracer", (render_options.width, render_options.height))
            .exit_on_esc(true)
            .opengl(opengl)
            .build()
            .unwrap();

    window.set_bench_mode(true);

    while let Some(e) = window.next() {
        let frame = render_frame(&scene, &camera, &render_options);

        let texture: G2dTexture = Texture::from_image(
            &mut window.factory,
            &frame,
            &TextureSettings::new()
        ).unwrap();

        window.draw_2d(&e, |c, g| {
            clear([1.0; 4], g);
            image(&texture, c.transform, g);
        });

        scene.spheres[0].center.z -= 0.01;
        scene.spheres[1].center.z -= 0.015;
    }
}
