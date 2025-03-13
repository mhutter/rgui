use std::time::Instant;

use rand::Rng;
use raylib::prelude::*;
use rgui_raylib::{conf, star::Star};

pub const APP: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"),);

fn main() {
    // Create the rendering window
    let (mut rl, thread) = raylib::init()
        .size(0, 0) // required to get fullscreen to take the full size
        .fullscreen()
        .undecorated()
        .title(APP)
        .build();
    rl.set_target_fps(conf::MAX_FRAMERATE);

    // Determine the screen size.
    // There is `get_monitor_width` etc., but they all only work after calling `raylib::init`.
    let window_size = rvec2(rl.get_screen_width(), rl.get_screen_height());

    // Load texture
    let texture = {
        let mut image = Image::load_image_from_mem(".png", include_bytes!("../res/star.png"))
            .expect("load image");
        let new_size = conf::STAR_RADIUS as i32 * 2;
        // Texture can only be scaled on drawing, so we have to configure the "1.0 size" here
        image.resize(new_size, new_size);
        image.gen_mipmaps();
        rl.load_texture_from_image(&thread, &image)
            .expect("load texture from image")
    };

    // Generate random stars; sort by depth
    let mut stars = create_stars(window_size, conf::STAR_COUNT, conf::FAR);
    stars.sort_unstable_by(|a, b| b.z.total_cmp(&a.z));

    // Preallocate our Vertex Array
    let mut va = vec![StarVertex::default(); conf::STAR_COUNT];

    let mut first = 0;
    while !rl.window_should_close() {
        let mut d = rl.begin_drawing(&thread);

        if d.is_key_pressed(KeyboardKey::KEY_Q) {
            break;
        }

        // Fake travel goes here
        for (i, s) in stars.iter_mut().enumerate().rev() {
            s.z -= conf::SPEED * conf::DT;
            if s.z < conf::NEAR {
                s.z = conf::FAR - (conf::NEAR - s.z);
                first = i;
            }
        }

        d.clear_background(Color::BLACK);

        for i in 0..stars.len() {
            // process stars by distance, farthest first
            let idx = (i + first) % stars.len();
            let star = &stars[idx];
            let vertex = va.get_mut(i).unwrap();

            // calculate the star's new distance
            update_geometry(star, vertex);
            // correctly position the star on the screen
            vertex.position += window_size * 0.5;
        }

        for v in va.iter() {
            d.draw_texture_ex(&texture, v.position, 0.0, v.scale, v.color);
        }
    }
}

#[derive(Clone, Copy, Default)]
pub struct StarVertex {
    position: Vector2,
    scale: f32,
    color: Color,
}

pub fn update_geometry(star: &Star, vertex: &mut StarVertex) {
    const RADIUS: Vector2 = Vector2 {
        x: conf::STAR_RADIUS,
        y: conf::STAR_RADIUS,
    };

    let scale = 1.0 / star.z;

    // Calculate the color based on the distance
    let depth_ratio = (star.z - conf::NEAR) / (conf::FAR - conf::NEAR);
    let color_ratio = 1.0 - depth_ratio;
    let c = (color_ratio * 255.0) as u8;
    let color = Color::new(c, c, c, 255);

    // Determine position and size based on the distance
    let p = (star.position - RADIUS) * scale;

    vertex.position = p;
    vertex.scale = scale;
    vertex.color = color;
}

/// Create a bunch of random stars
pub fn create_stars(window_size: Vector2, count: usize, scale: f32) -> Vec<Star> {
    let mut stars = Vec::with_capacity(count);
    let mut rng = rand::rng();

    // Determine the area that cannot contain any stars to prevent them from popping out of view
    // when they reach the viewport
    let window_world_size = window_size * conf::NEAR;
    let star_free_zone = Rectangle::new(
        window_world_size.x * -0.5,
        window_world_size.y * -0.5,
        window_world_size.x,
        window_world_size.y,
    );

    for _ in 0..count {
        loop {
            let x = (rng.random::<f32>() - 0.5) * window_size.x * scale;
            let y = (rng.random::<f32>() - 0.5) * window_size.y * scale;
            let z = rng.random::<f32>() * (conf::FAR - conf::NEAR) + conf::NEAR;
            let position = rvec2(x, y);

            if !star_free_zone.check_collision_point_rec(position) {
                stars.push(Star { position, z });
                break;
            }
        }
    }

    stars
}
