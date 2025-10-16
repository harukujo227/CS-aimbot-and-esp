use std::time::{Duration, Instant};

use glam::Vec2;
use rand::{Rng, rng};

use crate::{
    config::{Config, TriggerbotMode},
    cs2::{CS2, bones::Bones, player::Player, weapon_class::WeaponClass},
    math::angles_to_fov,
    mouse::Mouse,
};

#[derive(Debug, Default)]
pub struct Triggerbot {
    next_shot: Option<Instant>,
    previous_button_state: bool,
    pub(crate) active: bool,
    additional_shooting_end_time: Option<Instant>,
    next_additional_shot: Option<Instant>,
}

impl CS2 {
    pub fn triggerbot(&mut self, config: &Config) {
        let hotkey = config.aim.triggerbot_hotkey;
        let config = self.triggerbot_config(config);

        if !config.enabled || self.trigger.next_shot.is_some() {
            return;
        }

        let button_state = self.is_button_down(&hotkey);
        if config.mode == TriggerbotMode::Hold && !button_state {
            return;
        } else {
            if button_state && !self.trigger.previous_button_state {
                self.trigger.active = !self.trigger.active;
            }
            self.trigger.previous_button_state = button_state;
            if !self.trigger.active {
                return;
            }
        }

        let Some(local_player) = Player::local_player(self) else {
            return;
        };

        if config.flash_check && local_player.is_flashed(self) {
            return;
        }

        if config.scope_check
            && local_player.weapon_class(self) == WeaponClass::Sniper
            && !local_player.is_scoped(self)
        {
            return;
        }

        if config.velocity_check && local_player.velocity(self).length() > config.velocity_threshold
        {
            return;
        }

        let Some(player) = local_player.crosshair_entity(self) else {
            return;
        };

        if !self.is_ffa() && player.team(self) == local_player.team(self) {
            return;
        }

        if config.head_only {
            let head = player.bone_position(self, Bones::Head.u64());

            let target_angle = self.angle_to_target(&local_player, &head, &Vec2::ZERO);
            let view_angles = local_player.view_angles(self);
            let fov = angles_to_fov(&view_angles, &target_angle);

            let head_radius_fov =
                3.5 / (local_player.position(self) - player.position(self)).length() * 100.0;

            if fov > head_radius_fov {
                return;
            }
        }

        let mean = (*config.delay.start() + *config.delay.end()) as f32 / 2.0;
        let std_dev = (*config.delay.end() - *config.delay.start()) as f32 / 2.0;

        let normal = rand_distr::Normal::new(mean, std_dev).unwrap();
        use rand_distr::Distribution as _;
        let delay = normal.sample(&mut rng()).max(0.0) as u64;

        self.trigger.next_shot = Some(Instant::now() + Duration::from_millis(delay));

        if config.additional_duration_ms > 0 {
            self.trigger.additional_shooting_end_time = Some(
                Instant::now()
                    + Duration::from_millis(delay as u64 + config.additional_duration_ms as u64),
            );
        } else {
            self.trigger.additional_shooting_end_time = None;
        }
    }

    pub fn triggerbot_shoot(&mut self, mouse: &mut Mouse) {
        let now = Instant::now();

        if let Some(shot_time) = self.trigger.next_shot
            && now >= shot_time
        {
            mouse.left_press();
            mouse.left_release();
            self.trigger.next_shot = None;

            if self.trigger.additional_shooting_end_time.is_some() {
                let additional_delay = 50 + (rand::rng().random::<u64>() % 100);
                self.trigger.next_additional_shot =
                    Some(now + Duration::from_millis(additional_delay));
            }
        }

        if let Some(additional_shot_time) = self.trigger.next_additional_shot
            && let Some(end_time) = self.trigger.additional_shooting_end_time
            && now >= additional_shot_time
            && now < end_time
        {
            mouse.left_press();
            mouse.left_release();

            let next_delay = 40 + (rand::rng().random::<u64>() % 80);
            let next_shot_time = now + Duration::from_millis(next_delay);

            if next_shot_time < end_time {
                self.trigger.next_additional_shot = Some(next_shot_time);
            } else {
                self.trigger.next_additional_shot = None;
                self.trigger.additional_shooting_end_time = None;
            }
        }

        if let Some(end_time) = self.trigger.additional_shooting_end_time
            && now >= end_time
        {
            self.trigger.next_additional_shot = None;
            self.trigger.additional_shooting_end_time = None;
        }
    }
}
