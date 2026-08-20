use minifb::{Key, MouseMode, Window};
use nalgebra_glm::Vec2;
use std::f32::consts::PI;

use crate::maze::{is_wall, Maze};

pub struct Player {
    pub pos: Vec2,
    pub a: f32,
}

pub struct MouseState {
    last_x: f32,
    has_last: bool,
}

impl MouseState {
    pub fn new() -> Self {
        MouseState { last_x: 0.0, has_last: false }
    }
}

pub fn process_events(
    window: &Window,
    maze: &Maze,
    block_size: usize,
    player: &mut Player,
    mouse: &mut MouseState,
) {
    const MOVE_SPEED: f32 = 6.0;
    const ROTATION_SPEED: f32 = PI / 60.0;
    const MOUSE_SENSITIVITY: f32 = 0.003;
    // "grosor" del jugador en pixeles-mundo, para no pegarse a la pared
    // ni poder atravesarla por una esquina.
    const RADIUS: f32 = 15.0;

    if window.is_key_down(Key::A) || window.is_key_down(Key::Left) {
        player.a -= ROTATION_SPEED;
    }

    if window.is_key_down(Key::D) || window.is_key_down(Key::Right) {
        player.a += ROTATION_SPEED;
    }

    if let Some((mx, _my)) = window.get_mouse_pos(MouseMode::Clamp) {
        if mouse.has_last {
            let dx = mx - mouse.last_x;
            player.a += dx * MOUSE_SENSITIVITY;
        }
        mouse.last_x = mx;
        mouse.has_last = true;
    }

    let mut move_step = 0.0;
    if window.is_key_down(Key::W) || window.is_key_down(Key::Up) {
        move_step += MOVE_SPEED;
    }
    if window.is_key_down(Key::S) || window.is_key_down(Key::Down) {
        move_step -= MOVE_SPEED;
    }

    if move_step != 0.0 {
        let new_x = player.pos.x + move_step * player.a.cos();
        let new_y = player.pos.y + move_step * player.a.sin();


        if is_free(maze, new_x, player.pos.y, block_size, RADIUS) {
            player.pos.x = new_x;
        }
        if is_free(maze, player.pos.x, new_y, block_size, RADIUS) {
            player.pos.y = new_y;
        }
    }
}

fn is_free(maze: &Maze, x: f32, y: f32, block_size: usize, radius: f32) -> bool {
    let corners = [
        (x - radius, y - radius),
        (x + radius, y - radius),
        (x - radius, y + radius),
        (x + radius, y + radius),
    ];

    corners.iter().all(|&(px, py)| {
        if px < 0.0 || py < 0.0 {
            return false;
        }
        let i = px as usize / block_size;
        let j = py as usize / block_size;
        !is_wall(maze, j, i)
    })
}
