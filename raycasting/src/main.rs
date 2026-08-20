mod audio;
mod caster;
mod framebuffer;
mod maze;
mod player;
mod sprite;
mod text;

use minifb::{Key, KeyRepeat, Window, WindowOptions};
use nalgebra_glm::Vec2;
use std::f32::consts::PI;
use std::time::Instant;

use crate::audio::Audio;
use crate::caster::cast_ray;
use crate::framebuffer::Framebuffer;
use crate::maze::{height_in_cells, load_maze, width_in_cells, Maze};
use crate::player::{process_events, MouseState, Player};
use crate::sprite::{render_sprites, Sprite};
use crate::text::{draw_text, text_width};

const BLOCK_SIZE: usize = 100;
const FOV: f32 = PI / 3.0;
const SCREEN_W: usize = 1300;
const SCREEN_H: usize = 900;

const CEILING_COLOR: u32 = 0x333355;
const FLOOR_COLOR: u32 = 0x555544;

#[derive(PartialEq)]
enum GameState {
    Welcome,
    Playing,
    Success,
}

fn cell_color(cell: char) -> u32 {
    match cell {
        '+' => 0x00AAFF,
        '-' => 0xFF5555,
        '|' => 0xFFA500,
        '*' => 0x9932CC,
        'g' | 'G' => 0x00FF00,
        _ => 0xFFDDDD,
    }
}

fn shade(color: u32, factor: f32) -> u32 {
    let factor = factor.clamp(0.15, 1.0);
    let r = (((color >> 16) & 0xFF) as f32 * factor) as u32;
    let g = (((color >> 8) & 0xFF) as f32 * factor) as u32;
    let b = ((color & 0xFF) as f32 * factor) as u32;
    (r << 16) | (g << 8) | b
}

/// Dibuja techo, piso y una columna de pared por columna de pantalla.
/// Devuelve el buffer de profundidad (distancia corregida por columna)
/// para que los sprites sepan si quedan tapados por una pared.
fn render_3d(framebuffer: &mut Framebuffer, maze: &Maze, player: &Player) -> Vec<f32> {
    let width = framebuffer.width;
    let height = framebuffer.height;
    let mut depth = vec![f32::MAX; width];

    framebuffer.set_current_color(CEILING_COLOR);
    framebuffer.draw_rect(0, 0, width, height / 2);
    framebuffer.set_current_color(FLOOR_COLOR);
    framebuffer.draw_rect(0, height / 2, width, height - height / 2);

    for col in 0..width {
        let camera_x = 2.0 * col as f32 / width as f32 - 1.0;
        let angle = player.a + camera_x * (FOV / 2.0);

        let intersect = cast_ray(maze, player, angle, BLOCK_SIZE);
        let corrected = (intersect.distance * (angle - player.a).cos()).max(1.0);
        depth[col] = corrected;

        let wall_height = (BLOCK_SIZE as f32 * height as f32) / corrected;
        let half = height as f32 / 2.0;
        let draw_start = (half - wall_height / 2.0).max(0.0) as usize;
        let draw_end = (half + wall_height / 2.0).min(height as f32 - 1.0) as usize;

        let base_color = cell_color(intersect.impact);
        let fog = 1.0 - (corrected / 1400.0).min(0.8);
        framebuffer.set_current_color(shade(base_color, fog));
        if draw_start <= draw_end {
            framebuffer.draw_vertical_line(col, draw_start, draw_end);
        }
    }

    depth
}

fn render_minimap(framebuffer: &mut Framebuffer, maze: &Maze, player: &Player) {
    const SCALE: usize = 6;
    const MARGIN: usize = 12;

    let map_w = width_in_cells(maze) * SCALE;
    let map_h = height_in_cells(maze) * SCALE;
    if map_w + MARGIN * 2 > framebuffer.width || map_h + MARGIN * 2 > framebuffer.height {
        return;
    }

    let origin_x = framebuffer.width - map_w - MARGIN;
    let origin_y = MARGIN;

    framebuffer.set_current_color(0x000000);
    framebuffer.draw_rect(origin_x - 3, origin_y - 3, map_w + 6, map_h + 6);

    for (j, row) in maze.iter().enumerate() {
        for (i, &cell) in row.iter().enumerate() {
            if cell == ' ' {
                continue;
            }
            framebuffer.set_current_color(cell_color(cell));
            framebuffer.draw_rect(origin_x + i * SCALE, origin_y + j * SCALE, SCALE, SCALE);
        }
    }

    let px = origin_x + (player.pos.x / BLOCK_SIZE as f32 * SCALE as f32) as usize;
    let py = origin_y + (player.pos.y / BLOCK_SIZE as f32 * SCALE as f32) as usize;
    framebuffer.set_current_color(0xFF0000);
    framebuffer.draw_rect(px.saturating_sub(2), py.saturating_sub(2), 4, 4);

    for step in 0..6 {
        let x = (px as f32 + player.a.cos() * step as f32) as usize;
        let y = (py as f32 + player.a.sin() * step as f32) as usize;
        framebuffer.point(x, y);
    }
}

fn draw_centered(fb: &mut Framebuffer, y: usize, text: &str, color: u32, scale: usize) {
    let x = (fb.width / 2).saturating_sub(text_width(text, scale) / 2);
    draw_text(fb, x, y, text, color, scale);
}

/// Busca la primera celda con el caracter dado y devuelve su posicion
/// en coordenadas de mundo (centro de esa celda).
fn find_cell_world_pos(maze: &Maze, target: char) -> Option<Vec2> {
    for (j, row) in maze.iter().enumerate() {
        for (i, &c) in row.iter().enumerate() {
            if c == target || (target == 'g' && c == 'G') {
                let x = i as f32 * BLOCK_SIZE as f32 + BLOCK_SIZE as f32 / 2.0;
                let y = j as f32 * BLOCK_SIZE as f32 + BLOCK_SIZE as f32 / 2.0;
                return Some(Vec2::new(x, y));
            }
        }
    }
    None
}

fn main() {
    let (maze, mut player) = load_maze("./maze.txt", BLOCK_SIZE);

    // La antorcha animada se coloca justo en la meta, a modo de baliza
    // visual que ayuda a orientarse (ademas de cumplir el objetivo de
    // sprite animado).
    let mut sprites = match find_cell_world_pos(&maze, 'g') {
        Some(pos) => vec![Sprite::new(pos)],
        None => vec![],
    };

    let mut framebuffer = Framebuffer::new(SCREEN_W, SCREEN_H);
    framebuffer.set_background_color(CEILING_COLOR);

    let mut window = Window::new("Raycaster - CS UVG", SCREEN_W, SCREEN_H, WindowOptions::default())
        .expect("no se pudo abrir la ventana");
    window.set_cursor_visibility(false);

    let mut audio = Audio::new();
    let mut music_started = false;

    let mut mouse = MouseState::new();
    let mut last_frame = Instant::now();
    let mut fps: i32;
    let mut state = GameState::Welcome;

    while window.is_open() {
        if window.is_key_down(Key::Escape) {
            break;
        }

        let dt = last_frame.elapsed().as_secs_f32().max(0.0001);
        fps = (1.0 / dt) as i32;
        last_frame = Instant::now();

        match state {
            GameState::Welcome => {
                if window.is_key_pressed(Key::Enter, KeyRepeat::No) {
                    state = GameState::Playing;
                }
            }
            GameState::Playing => {
                if let Some(a) = audio.as_mut() {
                    if !music_started {
                        a.play_music("assets/music/background.mp3");
                        music_started = true;
                    }
                }

                process_events(&window, &maze, BLOCK_SIZE, &mut player, &mut mouse);

                let i = player.pos.x as usize / BLOCK_SIZE;
                let j = player.pos.y as usize / BLOCK_SIZE;
                if let Some(&cell) = maze.get(j).and_then(|row| row.get(i)) {
                    if cell == 'g' || cell == 'G' {
                        if let Some(a) = audio.as_ref() {
                            a.play_sfx("assets/sfx/success.wav");
                        }
                        if let Some(a) = audio.as_mut() {
                            a.stop_music();
                        }
                        music_started = false;
                        state = GameState::Success;
                    }
                }
            }
            GameState::Success => {
                if window.is_key_pressed(Key::Enter, KeyRepeat::No) {
                    // reinicia al spawn original del archivo
                    let (_, fresh_player) = load_maze("./maze.txt", BLOCK_SIZE);
                    player = fresh_player;
                    state = GameState::Welcome;
                }
            }
        }

        framebuffer.clear();

        match state {
            GameState::Welcome => {
                draw_centered(&mut framebuffer, 260, "RAYCASTER", 0xFFD700, 4);
                draw_centered(&mut framebuffer, 340, "CS UVG - GRAFICAS POR COMPUTADORA", 0xCCCCCC, 1);
                draw_centered(&mut framebuffer, 460, "PRESIONA ENTER PARA JUGAR", 0xFFFFFF, 2);
                draw_centered(&mut framebuffer, 520, "WASD MOVER - MOUSE/FLECHAS ROTAR - ESC SALIR", 0x999999, 1);
            }
            GameState::Playing => {
                for s in sprites.iter_mut() {
                    s.update(dt);
                }
                let depth = render_3d(&mut framebuffer, &maze, &player);
                render_sprites(&mut framebuffer, &sprites, &player, &depth, FOV, BLOCK_SIZE);
                render_minimap(&mut framebuffer, &maze, &player);

                draw_text(&mut framebuffer, 10, 10, &format!("FPS:{}", fps), 0x00FF00, 2);
            }
            GameState::Success => {
                draw_centered(&mut framebuffer, 300, "NIVEL COMPLETADO", 0x00FF00, 4);
                draw_centered(&mut framebuffer, 420, "PRESIONA ENTER PARA VOLVER AL MENU", 0xFFFFFF, 2);
            }
        }

        window
            .update_with_buffer(&framebuffer.buffer, SCREEN_W, SCREEN_H)
            .unwrap();
    }
}
