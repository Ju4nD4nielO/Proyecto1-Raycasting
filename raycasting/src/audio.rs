use rodio::{Decoder, DeviceSinkBuilder, MixerDeviceSink, Player};
use std::fs::File;
use std::io::BufReader;

/// Envoltorio sobre rodio para musica de fondo (en loop) y efectos de
/// sonido puntuales. 

pub struct Audio {
    // Debe vivir mientras el programa reproduzca algo: si se dropea,
    _device: MixerDeviceSink,
    music_player: Option<Player>,
}

impl Audio {
    pub fn new() -> Option<Self> {
        let device = DeviceSinkBuilder::open_default_sink().ok()?;
        Some(Audio { _device: device, music_player: None })
    }

    /// Arranca musica de fondo en loop infinito. 
    pub fn play_music(&mut self, path: &str) {
        if !std::path::Path::new(path).exists() {
            return;
        }
        let Ok(file) = File::open(path) else { return };
        let Ok(source) = Decoder::try_from(BufReader::new(file)) else { return };

        let player = Player::connect_new(self._device.mixer());
        player.append(rodio::source::Source::repeat_infinite(source));
        player.set_volume(0.4);
        self.music_player = Some(player);
    }

    pub fn stop_music(&mut self) {
        if let Some(player) = self.music_player.take() {
            player.stop();
        }
    }

    /// Reproduce un efecto de sonido una sola vez, sin bloquear el loop
    /// del juego (se reproduce en su propio Player desechable).
    pub fn play_sfx(&self, path: &str) {
        if !std::path::Path::new(path).exists() {
            return;
        }
        let Ok(file) = File::open(path) else { return };
        let Ok(source) = Decoder::try_from(BufReader::new(file)) else { return };

        let player = Player::connect_new(self._device.mixer());
        player.append(source);
        player.detach(); // se reproduce sola y se limpia cuando termina
    }
}
