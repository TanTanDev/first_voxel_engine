use super::direction::Direction;
use crate::color::Color;
use cgmath::Vector3;
use rand::Rng;

pub struct Quad {
    pub color: Color,
    pub direction: Direction,
    // in world position
    pub corners: [Vector3<f32>; 4],
}

const HALF_SIZE: f32 = 0.5f32;

impl Quad {
    pub fn from_direction(direction: Direction, pos: Vector3<f32>) -> Self {
        let corners = match direction {
            Direction::Left => [
                Vector3::new(pos.x - HALF_SIZE, pos.y - HALF_SIZE, pos.z - HALF_SIZE),
                Vector3::new(pos.x - HALF_SIZE, pos.y - HALF_SIZE, pos.z + HALF_SIZE),
                Vector3::new(pos.x - HALF_SIZE, pos.y + HALF_SIZE, pos.z + HALF_SIZE),
                Vector3::new(pos.x - HALF_SIZE, pos.y + HALF_SIZE, pos.z - HALF_SIZE),
            ],
            Direction::Right => [
                Vector3::new(pos.x - HALF_SIZE, pos.y + HALF_SIZE, pos.z - HALF_SIZE),
                Vector3::new(pos.x - HALF_SIZE, pos.y + HALF_SIZE, pos.z + HALF_SIZE),
                Vector3::new(pos.x - HALF_SIZE, pos.y - HALF_SIZE, pos.z + HALF_SIZE),
                Vector3::new(pos.x - HALF_SIZE, pos.y - HALF_SIZE, pos.z - HALF_SIZE),
            ],
            // assuming it's correct this is under i believe
            Direction::Down => [
                Vector3::new(pos.x - HALF_SIZE, pos.y - HALF_SIZE, pos.z - HALF_SIZE),
                Vector3::new(pos.x + HALF_SIZE, pos.y - HALF_SIZE, pos.z - HALF_SIZE),
                Vector3::new(pos.x + HALF_SIZE, pos.y - HALF_SIZE, pos.z + HALF_SIZE),
                Vector3::new(pos.x - HALF_SIZE, pos.y - HALF_SIZE, pos.z + HALF_SIZE),
            ],
            Direction::Up => [
                Vector3::new(pos.x - HALF_SIZE, pos.y - HALF_SIZE, pos.z + HALF_SIZE),
                Vector3::new(pos.x + HALF_SIZE, pos.y - HALF_SIZE, pos.z + HALF_SIZE),
                Vector3::new(pos.x + HALF_SIZE, pos.y - HALF_SIZE, pos.z - HALF_SIZE),
                Vector3::new(pos.x - HALF_SIZE, pos.y - HALF_SIZE, pos.z - HALF_SIZE),
            ],
            Direction::Back => [
                Vector3::new(pos.x - HALF_SIZE, pos.y - HALF_SIZE, pos.z - HALF_SIZE),
                Vector3::new(pos.x - HALF_SIZE, pos.y + HALF_SIZE, pos.z - HALF_SIZE),
                Vector3::new(pos.x + HALF_SIZE, pos.y + HALF_SIZE, pos.z - HALF_SIZE),
                Vector3::new(pos.x + HALF_SIZE, pos.y - HALF_SIZE, pos.z - HALF_SIZE),
            ],
            Direction::Forward => [
                Vector3::new(pos.x + HALF_SIZE, pos.y - HALF_SIZE, pos.z - HALF_SIZE),
                Vector3::new(pos.x + HALF_SIZE, pos.y + HALF_SIZE, pos.z - HALF_SIZE),
                Vector3::new(pos.x - HALF_SIZE, pos.y + HALF_SIZE, pos.z - HALF_SIZE),
                Vector3::new(pos.x - HALF_SIZE, pos.y - HALF_SIZE, pos.z - HALF_SIZE),
            ],
        };

        let green_range = 0.4f32;
        Self {
            corners,
            //color: color::colors::GREEN,
            //color: Color::new(0.7f32, (1.-green_range) + rand::thread_rng().gen_range(0f32..green_range), 0.3f32, 1.),
            color: Color::new(
                rand::thread_rng().gen_range(0f32..0.1f32),
                (1. - green_range) + rand::thread_rng().gen_range(0f32..green_range),
                rand::thread_rng().gen_range(0f32..0.1f32),
                1.,
            ),
            direction,
        }
    }
}
