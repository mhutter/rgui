use sfml::system::Vector2f;

pub struct Star {
    pub position: Vector2f,
    pub z: f32,
}

impl Star {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self {
            position: Vector2f::new(x, y),
            z,
        }
    }
}
