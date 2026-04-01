use crate::{
    config::Config,
    cs2::{CS2, entity::player::Player},
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

        let _weapon = local_player.weapon(self);
        let Some(_weapon_gsn) = local_player.weapon_game_scene_node(self) else {
            return;
        };

        // NetworkGameClient_DeltaTick (0x??) = -1

        // weapon address -> attribute manager + item +

        // econ entity (weapon) -> attribute manager, original owner xuid low/high,
        // fallback paint kit/wear
    }
}
