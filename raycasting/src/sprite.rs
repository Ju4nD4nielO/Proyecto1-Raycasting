use nalgebra_glm::Vec2;

use crate::framebuffer::Framebuffer;
use crate::player::Player;

const FLAME_PALETTE: [u32; 4] = [0xFF8C00, 0xFFA500, 0xFFD700, 0xFF6600];

pub struct Sprite {
    pub pos: Vec2,
    time: f32,
}

impl Sprite {
    pub fn new(pos: Vec2) -> Self {
        Sprite { pos, time: 0.0 }
    }

    pub fn update(&mut self, dt: f32) {
        self.time += dt;
    }

    fn current_color(&self) -> u32 {
        let frame = ((self.time * 6.0) as usize) % FLAME_PALETTE.len();
        FLAME_PALETTE[frame]
    }

    /// Factor de escala que hace que la antorcha "respire"
    fn pulse(&self) -> f32 {
        1.0 + 0.15 * (self.time * 5.0).sin()
    }
}

pub fn render_sprites(
    fb: &mut Framebuffer,
    sprites: &[Sprite],
    player: &Player,
    depth: &[f32],
    fov: f32,
    block_size: usize,
) {
    for sprite in sprites {
        let dx = sprite.pos.x - player.pos.x;
        let dy = sprite.pos.y - player.pos.y;
        let dist = (dx * dx + dy * dy).sqrt().max(1.0);

        
        let angle_to_sprite = dy.atan2(dx);
        let mut relative = angle_to_sprite - player.a;
        while relative > std::f32::consts::PI {
            relative -= 2.0 * std::f32::consts::PI;
        }
        while relative < -std::f32::consts::PI {
            relative += 2.0 * std::f32::consts::PI;
        }

        if relative.abs() > fov / 2.0 + 0.3 {
            continue; // fuera del campo de vision, ni lo calculamos
        }

        let corrected_dist = (dist * relative.cos()).max(1.0);

        let screen_x = ((relative / fov) + 0.5) * fb.width as f32;
        let sprite_size = ((block_size as f32 * fb.height as f32) / corrected_dist) * sprite.pulse();

        let half = sprite_size / 2.0;
        let x_start = (screen_x - half).max(0.0) as usize;
        let x_end = (screen_x + half).min(fb.width as f32 - 1.0) as usize;
        let y_center = fb.height as f32 / 2.0;
        let y_start = (y_center - half).max(0.0) as usize;
        let y_end = (y_center + half).min(fb.height as f32 - 1.0) as usize;

        fb.set_current_color(sprite.current_color());
        for x in x_start..=x_end.max(x_start) {
            // si la pared en esa columna esta mas cerca que el sprite,
            // esa columna del sprite queda tapada (prueba de profundidad)
            if depth.get(x).copied().unwrap_or(f32::MAX) < corrected_dist {
                continue;
            }
            fb.draw_rect(x, y_start, 1, y_end.saturating_sub(y_start).max(1));
        }
    }
}
