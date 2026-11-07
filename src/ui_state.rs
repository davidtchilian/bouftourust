use crate::simulation::Phase;
use bevy::prelude::*;
#[derive(Resource)]
pub struct UiState {
    pub inventory: bool,
    pub dungeons: bool,
    pub help: bool,
    pub selected: Option<String>,
    pub filter: String,
    pub direction: usize,
    pub reduced_motion: bool,
    pub aim: Vec2,
    pub configured: bool,
    pub fps: bool,
    pub manual_pause: bool,
    pub input_frame: Option<u64>,
    pub movement_stopped: bool,
}
impl Default for UiState {
    fn default() -> Self {
        Self {
            inventory: false,
            dungeons: false,
            help: true,
            selected: None,
            filter: String::new(),
            direction: 7,
            reduced_motion: false,
            aim: Vec2::new(400., -200.),
            configured: false,
            fps: false,
            manual_pause: false,
            input_frame: None,
            movement_stopped: false,
        }
    }
}

impl UiState {
    pub fn escape(&mut self, phase: Phase) {
        if self.inventory || self.dungeons || self.help {
            self.inventory = false;
            self.dungeons = false;
            self.help = false;
        } else if phase == Phase::Combat {
            self.manual_pause = !self.manual_pause;
        }
    }
    pub fn pauses_game(&self) -> bool {
        self.inventory || self.dungeons || self.manual_pause
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn playing() -> UiState {
        UiState {
            help: false,
            ..default()
        }
    }
    #[test]
    fn escape_closes_dialog_before_pausing() {
        for inventory in [true, false] {
            let mut s = playing();
            s.inventory = inventory;
            s.dungeons = !inventory;
            s.escape(Phase::Combat);
            assert!(!s.inventory && !s.dungeons);
            assert!(!s.pauses_game());
        }
    }
    #[test]
    fn escape_toggles_combat_pause() {
        let mut s = playing();
        s.escape(Phase::Combat);
        assert!(s.pauses_game());
        s.escape(Phase::Combat);
        assert!(!s.pauses_game());
    }
    #[test]
    fn dialog_closure_preserves_manual_pause() {
        let mut s = playing();
        s.manual_pause = true;
        s.inventory = true;
        s.escape(Phase::Combat);
        assert!(!s.inventory);
        assert!(s.pauses_game());
    }
    #[test]
    fn escape_closes_help_without_pausing() {
        let mut s = UiState::default();
        s.escape(Phase::Combat);
        assert!(!s.help);
        assert!(!s.pauses_game());
    }
    #[test]
    fn escape_does_not_pause_outside_combat() {
        for phase in [Phase::Hub, Phase::Victory, Phase::Death] {
            let mut s = playing();
            s.escape(phase);
            assert!(!s.pauses_game());
        }
    }
}
