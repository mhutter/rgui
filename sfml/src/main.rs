use rand::Rng;
use rgui_sfml::{conf, events::process_events, star::Star};
use sfml::{
    graphics::{
        Color, FloatRect, PrimitiveType, RenderStates, RenderTarget, RenderWindow, Texture,
        Transform, Vertex,
    },
    system::Vector2f,
    window::{Style, VideoMode},
};

const APP: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"),);

fn main() {
    // Determine the screen size
    let window_size = VideoMode::fullscreen_modes().first().unwrap();

    // Create the window to work in
    let mut window = RenderWindow::new(*window_size, APP, Style::FULLSCREEN, &Default::default())
        .expect("open window");
    window.set_framerate_limit(conf::MAX_FRAMERATE);
    window.set_mouse_cursor_visible(false);

    let window_size = Vector2f::new(window_size.width as _, window_size.height as _);

    // Load texture
    let texture = {
        let mut t = Texture::new().expect("texture");
        t.load_from_memory(include_bytes!("../res/star.png"), Default::default())
            .unwrap();
        t.set_smooth(true);
        t.generate_mipmap().unwrap();
        t
    };

    // Generate random stars and order by depth
    //
    // Farthest first, closest last.
    let mut stars = create_stars(window_size, conf::STAR_COUNT, conf::FAR);
    stars.sort_unstable_by(|a, b| b.z.total_cmp(&a.z));

    // Create Vertex Array and pre-fill texture coordinates
    //
    // The Vertex Array contains the coordinates of the four edges of the texture, for each of our
    // stars.
    let texture_size: Vector2f = texture.size().as_other();
    let texture_coords = [
        Vector2f::new(0.0, 0.0),
        Vector2f::new(texture_size.x, 0.0),
        Vector2f::new(texture_size.x, texture_size.y),
        Vector2f::new(0.0, texture_size.y),
    ];
    let mut va: Vec<_> = (0..conf::STAR_COUNT)
        .flat_map(|_| {
            // create a Vertex for each `texture_coords`
            texture_coords.map(|tex_coords| Vertex {
                tex_coords,
                ..Default::default()
            })
        })
        .collect();

    // Precalculate the RenderState
    let render_state = {
        let mut tf = Transform::default();
        let t = window_size * 0.5;
        tf.translate(t.x, t.y);
        RenderStates {
            transform: tf,
            texture: Some(&texture),
            ..Default::default()
        }
    };

    // Index of the first (== farthest away from the viewpoint) star
    let mut first = 0;
    while window.is_open() {
        process_events(&mut window);

        // Fake travel
        //
        // Process stars from farthest to closest.
        for (i, s) in stars.iter_mut().enumerate().rev() {
            s.z -= conf::SPEED * conf::DT;
            if s.z < conf::NEAR {
                s.z = conf::FAR - (conf::NEAR - s.z);
                // This star is now the first. Since we process from farthest to closest, this will
                // now always be the index of the farthest star.
                first = i;
            }
        }

        window.clear(Color::BLACK);

        for i in 0..stars.len() {
            // process stars by distance, farthest first.
            let idx = (i + first) % stars.len();
            let star = &stars[idx];

            // Calculate the star's new position, size and color
            update_geometry(i, star, &mut va);
        }

        // Batch draw all stars
        window.draw_primitives(&va, PrimitiveType::QUADS, &render_state);

        window.display();
    }
}

/// Calculate a Star's position on the screen based on its coordinates and distance.
///
/// Store the result in `va`.
#[allow(clippy::identity_op)]
pub fn update_geometry(idx: usize, star: &Star, va: &mut [Vertex]) {
    let scale = 1.0 / star.z;

    // Calculate the color based on the distance
    let depth_ratio = (star.z - conf::NEAR) / (conf::FAR - conf::NEAR);
    let color_ratio = 1.0 - depth_ratio;
    let c = (color_ratio * 255.0) as u8;
    let color = Color::rgb(c, c, c);

    // Determine position and size based on the distance
    let p = star.position * scale;
    let r = conf::STAR_RADIUS * scale;

    let i = idx * 4;

    va[i + 0].position = Vector2f::new(p.x - r, p.y - r);
    va[i + 1].position = Vector2f::new(p.x + r, p.y - r);
    va[i + 2].position = Vector2f::new(p.x + r, p.y + r);
    va[i + 3].position = Vector2f::new(p.x - r, p.y + r);

    va[i + 0].color = color;
    va[i + 1].color = color;
    va[i + 2].color = color;
    va[i + 3].color = color;
}

/// Create a bunch of random stars
pub fn create_stars(window_size: Vector2f, count: usize, scale: f32) -> Vec<Star> {
    let mut stars = Vec::with_capacity(count);
    let mut rng = rand::rng();

    // Determine the area that cannot contain any stars to prevent them from popping out of view
    // when they reach the viewport
    let window_world_size = window_size * conf::NEAR;
    let star_free_zone = FloatRect::from_vecs(-window_world_size * 0.5, window_world_size);

    for _ in 0..count {
        loop {
            let x = (rng.random::<f32>() - 0.5) * window_size.x * scale;
            let y = (rng.random::<f32>() - 0.5) * window_size.y * scale;
            let z = rng.random::<f32>() * (conf::FAR - conf::NEAR) + conf::NEAR;

            if !star_free_zone.contains2(x, y) {
                stars.push(Star::new(x, y, z));
                break;
            }
        }
    }

    stars
}
