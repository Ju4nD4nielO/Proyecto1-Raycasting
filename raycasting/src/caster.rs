use crate::maze::{is_wall, Maze};
use crate::player::Player;

/// Resultado de lanzar un rayo: que tan lejos viajo antes de pegarle a
/// una pared y que caracter tenia esa celda (para poder colorearla
/// distinto segun el tipo de pared).
pub struct Intersect {
    pub distance: f32,
    pub impact: char,
}

/// Lanza un rayo desde la posicion del jugador en el angulo `a`,
/// avanzando de a 1.0 unidad (misma tecnica que ya tenias) hasta
/// encontrar una celda que no sea espacio vacio.
pub fn cast_ray(maze: &Maze, player: &Player, a: f32, block_size: usize) -> Intersect {
    let mut d: f32 = 0.0;

    loop {
        let x = player.pos.x + d * a.cos();
        let y = player.pos.y + d * a.sin();

        if x < 0.0 || y < 0.0 {
            return Intersect { distance: d.max(1.0), impact: '#' };
        }

        let i = x as usize / block_size;
        let j = y as usize / block_size;

        if is_wall(maze, j, i) {
            let impact = maze.get(j).and_then(|row| row.get(i)).copied().unwrap_or('#');
            return Intersect { distance: d.max(1.0), impact };
        }

        d += 1.0;
    }
}
