mod event;

use super::*;

impl Model {
    pub fn update(&mut self, delta_time: Time) {
        self.real_time += delta_time;

        self.update_texts(delta_time);
        self.update_cursor(delta_time);
        self.control(delta_time);
        self.update_player(delta_time);
        self.update_weapon(delta_time);
    }

    fn update_cursor(&mut self, _delta_time: Time) {
        // Validate cursor state (in case some event is missed, e.g. when window loses focus)
        let attack =
            geng_utils::key::is_key_pressed(self.geng.window(), &self.config.controls.attack);
        let defend =
            geng_utils::key::is_key_pressed(self.geng.window(), &self.config.controls.defend);
        match self.player.cursor.state {
            CursorState::Idle => {
                if attack {
                    self.player.cursor.state = CursorState::Attack;
                } else if defend {
                    self.player.cursor.state = CursorState::Defend;
                }
            }
            CursorState::Attack => {
                if !attack {
                    self.player.cursor.state = CursorState::Idle;
                }
            }
            CursorState::Defend => {
                if !defend {
                    self.player.cursor.state = CursorState::Idle;
                }
            }
        }

        if self.player.cursor.state != self.player.cursor.last_state {
            self.check_action();
            self.player.cursor.last_state = self.player.cursor.state;
        }

        self.player
            .cursor
            .history
            .retain(|entry| self.real_time - entry.time < self.config.cursor.trail_time);
    }

    fn control(&mut self, _delta_time: Time) {
        // Update weapon action
        if !self.player.weapon.action.swinging() {
            let start = self
                .player
                .cursor
                .history
                .iter()
                .rev()
                .position(|entry| entry.state != self.player.cursor.last_state)
                .map(|len| self.player.cursor.history.len() - len)
                .unwrap_or(0);
            let target = if let Some(start) = self.player.cursor.history.get(start) {
                start.relative_pos
            } else {
                self.player.weapon.position
            };
            self.player.weapon.action = match self.player.cursor.state {
                CursorState::Idle => WeaponAction::Idle {
                    target: self.player.cursor.pos,
                },
                CursorState::Attack => WeaponAction::Charging {
                    target,
                    intent: WeaponIntent::Attack,
                },
                CursorState::Defend => WeaponAction::Charging {
                    target,
                    intent: WeaponIntent::Defend,
                },
            };
        }

        let mut move_dir = vec2::<f32>::ZERO;
        if geng_utils::key::is_key_pressed(self.geng.window(), &self.config.controls.up) {
            move_dir.y += 1.0;
        }
        if geng_utils::key::is_key_pressed(self.geng.window(), &self.config.controls.down) {
            move_dir.y -= 1.0;
        }
        if geng_utils::key::is_key_pressed(self.geng.window(), &self.config.controls.left) {
            move_dir.x -= 1.0;
        }
        if geng_utils::key::is_key_pressed(self.geng.window(), &self.config.controls.right) {
            move_dir.x += 1.0;
        }
        self.player.target_move_dir = move_dir.as_r32();
    }

    fn update_player(&mut self, delta_time: Time) {
        let target_velocity = self.player.target_move_dir * self.config.player.walk_speed;
        self.player.velocity += (target_velocity - self.player.velocity)
            .clamp_len(..=self.config.player.acceleration * delta_time);

        self.player.position += self.player.velocity * delta_time;
    }

    fn update_weapon(&mut self, delta_time: Time) {
        let weapon = &mut self.player.weapon;
        match &weapon.action {
            WeaponAction::Swing(swing) => {
                let t = swing.arc.project(weapon.position);
                if t > R32::ONE {
                    // Motion finished - boost backwards
                    let boost = (self.player.cursor.pos - weapon.position) * r32(5.0) * swing.power;
                    weapon.velocity = (weapon.velocity + boost).clamp_len(..=weapon.speed_max);
                    weapon.action = WeaponAction::Idle {
                        target: self.player.cursor.pos,
                    };
                } else {
                    let projection = swing.arc.get(t);
                    let tangent = swing.arc.tangent(t);
                    let normal = projection - weapon.position;

                    let target_vel = (normal * r32(5.0)
                        + (tangent.normalize_or_zero() * r32(5.0) * swing.power))
                        * r32(3.0);
                    let target_vel = target_vel.clamp_len(..=r32(1.5) * weapon.speed_max);

                    weapon.velocity += (target_vel - weapon.velocity)
                        .clamp_len(..=weapon.acceleration * delta_time);
                }
            }
            WeaponAction::Idle { target } => {
                let target = target.clamp_len(..=weapon.reach);
                let target_vel =
                    ((target - weapon.position) * r32(10.0)).clamp_len(..=weapon.speed_max);
                weapon.velocity +=
                    (target_vel - weapon.velocity).clamp_len(..=weapon.acceleration * delta_time);
            }
            WeaponAction::Charging { target, intent } => {
                let target = target.clamp_len(..=weapon.reach);
                let target_vel =
                    ((target - weapon.position) * r32(10.0)).clamp_len(..=weapon.speed_max);
                weapon.velocity +=
                    (target_vel - weapon.velocity).clamp_len(..=weapon.acceleration * delta_time);
            }
        }
        weapon.position =
            (weapon.position + weapon.velocity * delta_time).clamp_len(..=weapon.reach);

        weapon
            .history
            .retain(|entry| self.real_time - entry.time < self.config.cursor.trail_time);
        weapon.history.push_back(CursorEntry {
            world_pos: self.player.position + weapon.position,
            relative_pos: weapon.position,
            time: self.real_time,
            state: match &weapon.action {
                WeaponAction::Swing(WeaponSwing { intent, .. }) => match intent {
                    WeaponIntent::Attack => CursorState::Attack,
                    WeaponIntent::Defend => CursorState::Defend,
                },
                _ => CursorState::Idle,
            },
        });
    }

    pub(super) fn check_action(&mut self) {
        let cursor = &mut self.player.cursor;
        let Some(end) = cursor
            .history
            .iter()
            .rev()
            .position(|entry| entry.state == cursor.last_state)
        else {
            return;
        };

        let start = cursor
            .history
            .iter()
            .rev()
            .skip(end)
            .position(|entry| entry.state != cursor.last_state)
            .map(|len| cursor.history.len() - end - len)
            .unwrap_or(0);
        let end = cursor.history.len() - 1 - end;

        let mid = cursor.history[(start + end) / 2];
        let start = cursor.history[start];
        let end = cursor.history[end];

        let pos = self.player.position + (start.relative_pos + end.relative_pos) / r32(2.0);

        let time = (end.time - start.time) / self.config.cursor.trail_time;

        let power_t = time; // Maybe sqrt
        let power = power_t * (self.config.weapon.power_max - self.config.weapon.power_min)
            + self.config.weapon.power_min;

        let (intent, text) = match end.state {
            CursorState::Idle => return,
            CursorState::Attack => (WeaponIntent::Attack, "Slash"),
            CursorState::Defend => (WeaponIntent::Defend, "Parry"),
        };

        log::debug!(
            "{} at ({:.02}, {:.02}), power: {:.02}",
            text,
            pos.x,
            pos.y,
            power
        );

        let weapon = &mut self.player.weapon;
        let arc = Parabola::new([start.relative_pos, mid.relative_pos, end.relative_pos]);
        weapon.action = WeaponAction::Swing(WeaponSwing { intent, power, arc });
        // Boost
        let t = arc.project(weapon.position);
        let projection = arc.get(t);
        let tangent = arc.tangent(t);
        let normal = projection - weapon.position;
        let boost =
            (normal * r32(3.0) + (tangent.normalize_or_zero() * r32(5.0) * power)) * r32(3.0);
        weapon.velocity = (weapon.velocity + boost).clamp_len(..=weapon.speed_max);

        let degrees = r32(thread_rng().gen_range(-15.0..=15.0));
        self.floating_texts.push(FloatingText {
            text: format!("{} {}", text, power.round()),
            pos,
            lifetime: Bounded::new_max(r32(0.5)),
            initial_scale: r32(1.0),
            rotation: Angle::from_degrees(degrees),
        });
    }

    fn update_texts(&mut self, delta_time: Time) {
        for text in &mut self.floating_texts {
            text.lifetime.change(-delta_time);
        }
        self.floating_texts
            .retain(|text| text.lifetime.is_above_min());
    }
}
