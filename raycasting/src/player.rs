use minifb::{Key, MouseMode, Window};
use nalgebra_glm::Vec2;
use std::f32::consts::PI;

use crate::maze::{is_wall, Maze};

pub struct Player {
    pub pos: Vec2,
    pub a: f32,
}

pub fn process_events(window: &Window, maze: &Maze, block_size: usize, player: &mut Player) {
    const MOVE_SPEED: f32 = 6.0;
    const ROTATION_SPEED: f32 = PI / 30.0;
    // "grosor" del jugador en pixeles-mundo, para no pegarse a la pared
    // ni poder atravesarla por una esquina.
    const RADIUS: f32 = 15.0;

    if window.is_key_down(Key::A) || window.is_key_down(Key::Left) {
        player.a -= ROTATION_SPEED;
    }
    if window.is_key_down(Key::D) || window.is_key_down(Key::Right) {
        player.a += ROTATION_SPEED;
    }

    apply_mouse_look(window, player);

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

        // Se prueba cada eje por separado para poder "deslizar" contra
        // la pared en vez de trabarse en las esquinas.
        if is_free(maze, new_x, player.pos.y, block_size, RADIUS) {
            player.pos.x = new_x;
        }
        if is_free(maze, player.pos.x, new_y, block_size, RADIUS) {
            player.pos.y = new_y;
        }
    }
}

/// Rotacion horizontal con el mouse, estilo "joystick/palanca" 
fn apply_mouse_look(window: &Window, player: &mut Player) {
    const DEADZONE: f32 = 0.15; // 15% central de la ventana no gira, para poder mirar quieto
    const MAX_TURN_PER_FRAME: f32 = 0.045; 

    let (width, _height) = window.get_size();
    if width == 0 {
        return;
    }

    if let Some((mx, _my)) = window.get_mouse_pos(MouseMode::Clamp) {
        let center_x = width as f32 / 2.0;
        // normalized va de -1.0 (borde izquierdo) a 1.0 (borde derecho)
        let normalized = (mx - center_x) / center_x;

        if normalized.abs() > DEADZONE {
            // reescala lo que esta fuera de la deadzone a 0..1 para que
            // el giro empiece suave justo al salir de la zona muerta
            let sign = normalized.signum();
            let magnitude = (normalized.abs() - DEADZONE) / (1.0 - DEADZONE);
            player.a += sign * magnitude * MAX_TURN_PER_FRAME;
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
