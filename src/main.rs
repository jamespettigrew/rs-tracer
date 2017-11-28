extern crate cgmath;
extern crate image as im;
extern crate piston_window;

use cgmath::{InnerSpace, Point3, Vector3};
use im::{Rgba, RgbaImage};
use piston_window::*;
use std::fmt;
use std::io::{self, Write};
use std::time::Instant;

struct Sphere {
    center: Point3<f32>,
    radius: f32,
}

impl Sphere {
    fn intersects(&self, ray: &Ray) -> Option<f32> {
        //This method has next to no effect on fps
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

    fn normal(&self, surface_point: Point3<f32>) -> Vector3<f32> {
        surface_point - self.center
    }
}

struct Ray {
    origin: Point3<f32>,
    direction: Vector3<f32>,
}

struct Camera {
    position: Point3<f32>,
    up: Vector3<f32>,
    at: Vector3<f32>,
    fov: f32,
}


struct RenderOptions {
    width: u32,
    height: u32,
}

struct Scene {
    spheres: Vec<Sphere>,
}

struct Fps {
    a: u32,
    b: u32,
    c: u32,
    old: Instant,
}

impl Fps {
    fn tick(&mut self) {
        let now = Instant::now();
        self.c = self.b;
        self.b = self.a;
        self.a = now.duration_since(self.old).subsec_nanos();
        self.old = now;
    }
}

impl fmt::Display for Fps {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let fps = 1000000000.0 / (((self.a + self.b + self.c) / 3) as f64);
        write!(f, "\r {:.2} fps", fps)
    }
}

fn closest_intersection<'a>(scene: &'a Scene, ray: &Ray) -> Option<(&'a Sphere, f32)> {
    scene
        .spheres
        .as_slice()
        .into_iter()
        .filter_map(|s| {
            let intersection = s.intersects(ray);
            match intersection {
                Some(i) => {
                    return if i.is_nan() { None } else { Some((s, i)) };
                }
                None => None,
            }
        })
        .min_by(|x, y| {
            let &(s1, i1) = x;
            let &(s2, i2) = y;
            return i1.partial_cmp(&i2).unwrap(); // Shouldn't ever hit NaN due to check above
        })
}

fn get_pixel_color(scene: &Scene, ray: &Ray) -> Rgba<u8> {
    let closest_intersection = closest_intersection(&scene, ray);
    match closest_intersection {
        Some(i) => {
            let (sphere, ray_distance) = i;
            let intersection_point = ray.origin + (ray.direction * ray_distance);
            let normal = sphere.normal(intersection_point);
            let facing_ratio = 0f32.max(normal.dot(-ray.direction));
            let shade: u8 = (255.0 * facing_ratio) as u8;
            return Rgba([shade, shade, shade, 255]);
        }
        None => Rgba([0, 0, 0, 255]),
    }
}

fn render_frame(
    scene: &Scene,
    camera: &Camera,
    render_options: &RenderOptions,
    img: &mut RgbaImage,
) {
    let theta = camera.fov.to_radians() / 2.0;
    let fov_scalar = theta.tan();
    let w = render_options.width as f32;
    let h = render_options.height as f32;
    let aspect_ratio = w / h;
    let mut px_x = 0;
    let mut px_y = 0;
    loop {
        if px_y >= render_options.height {
            px_y = 0;
            px_x = px_x + 1;
        }
        if px_x >= render_options.width {
            return;
        }

        // Calculate pixel NDC (normalized device coordinates)
        let px_ndc_x = ((px_x as f32) + 0.5) / w;
        let px_ndc_y = ((px_y as f32) + 0.5) / h;

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
        let ray = Ray {
            origin: camera.position,
            direction: ray_vector,
        };

        let color = get_pixel_color(scene, &ray);
        img.put_pixel(px_x, px_y, color);
        px_y = px_y + 1;
    }
}


fn main() {
    let mut spheres = Vec::new();
    spheres.push(Sphere {
        center: Point3 {
            x: -2.0,
            y: 0.0,
            z: -4.0,
        },
        radius: 1.0,
    });
    spheres.push(Sphere {
        center: Point3 {
            x: 4.0,
            y: 2.0,
            z: -10.0,
        },
        radius: 0.9,
    });

    let mut scene = Scene { spheres: spheres };

    let camera = Camera {
        position: Point3 {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        },
        up: Vector3 {
            x: 0.0,
            y: 1.0,
            z: 0.0,
        },
        at: Vector3 {
            x: 1.0,
            y: 0.0,
            z: 0.0,
        },
        fov: 90.0,
    };
    let render_options = RenderOptions {
        width: 640,
        height: 640,
    };

    let opengl = OpenGL::V3_2;
    let mut window: PistonWindow =
        WindowSettings::new("rs-tracer", (render_options.width, render_options.height))
            .exit_on_esc(true)
            .opengl(opengl)
            .build()
            .unwrap();

    window.set_bench_mode(true);
    let mut fps = Fps {
        a: 0,
        b: 0,
        c: 0,
        old: Instant::now(),
    };

    let mut frame = RgbaImage::new(render_options.width, render_options.height);
    while let Some(e) = window.next() {
        //Removing render frame gives ~300x fps
        render_frame(&scene, &camera, &render_options, &mut frame);

        fps.tick();
        print!("{}", fps);
        let _ = io::stdout().flush(); // Don't care if flush fails

        let texture: G2dTexture =
            Texture::from_image(&mut window.factory, &frame, &TextureSettings::new()).unwrap();

        window.draw_2d(&e, |c, g| {
            clear([1.0; 4], g);
            image(&texture, c.transform, g);
        });

        scene.spheres[0].center.z -= 0.01;
        scene.spheres[1].center.z -= 0.015;
    }
}
