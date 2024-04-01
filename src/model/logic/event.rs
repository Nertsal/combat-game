use super::*;

impl Model {
    pub fn handle_event(&mut self, event: Event) {
        match event {
            Event::CursorMove { delta } => {
                let position = self.player.cursor.pos + delta;

                // Clamp by reach
                let position = position.clamp_len(..=self.player.weapon.reach);
                self.player.cursor.pos = position;

                self.player.cursor.history.push_back(CursorEntry {
                    world_pos: self.player.position + self.player.cursor.pos,
                    relative_pos: self.player.cursor.pos,
                    time: self.real_time,
                    state: self.player.cursor.state,
                });
            }
            Event::Charge(intent) => {
                self.player.cursor.state = match intent {
                    WeaponIntent::Attack => CursorState::Attack,
                    WeaponIntent::Defend => CursorState::Defend,
                };
            }
            Event::Release => {
                self.player.cursor.state = CursorState::Idle;
            }
        }
    }
}
