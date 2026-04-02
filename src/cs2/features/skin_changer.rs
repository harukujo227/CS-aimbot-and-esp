use crate::{
    config::Config,
    cs2::{
        CS2,
        entity::{player::Player, weapon::Weapon},
    },
};

impl CS2 {
    #[allow(dead_code)]
    pub fn skin_changer(&self, config: &Config) {
        if !config.misc.skin_changer {
            return;
        }

        let Some(local_player) = Player::local_player(self) else {
            return;
        };

        let weapon = local_player.weapon(self);
        if weapon != Weapon::Ak47 {
            return;
        }

        let weapon_address = local_player.weapon_address(self);
        if weapon_address == 0 {
            return;
        };

        let changed = Weapon::apply_skin(weapon_address, self);
        if changed {
            local_player.update_view_model(self, weapon_address);
            self.process
                .write(self.offsets.direct.network_client + 0x25C, -1);
        }

        // NetworkGameClient_DeltaTick (0x??) = -1

        // weapon address -> attribute manager + item +

        // econ entity (weapon) -> attribute manager, original owner xuid low/high,
        // fallback paint kit/wear
    }
}
