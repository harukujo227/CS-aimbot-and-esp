use std::{
    cell::RefCell,
    collections::HashMap,
    sync::{Arc, Mutex},
};

use glam::{IVec2, Mat4, Vec2, Vec3};

use crate::{
    bvh::Bvh,
    config::{AimbotConfig, Config, KeyMode, RcsConfig, TriggerbotConfig},
    constants::cs2::{self, TEAM_CT, TEAM_T, class},
    cs2::{
        bones::Bones,
        entity::{
            Entity, EntityInfo, GrenadeInfo, inferno::Inferno, molotov::Molotov,
            planted_c4::PlantedC4, player::Player, smoke::Smoke, weapon::Weapon,
        },
        features::{aimbot::Aimbot, esp_toggle::EspToggle, rcs::Recoil, triggerbot::Triggerbot},
        offsets::Offsets,
        target::Target,
    },
    data::{Data, PlayerData},
    game::Game,
    key_codes::KeyCode,
    math::{angles_from_vector, vec2_clamp},
    os::{mouse::Mouse, process::Process},
};

pub mod bones;
pub mod entity;
mod features;
mod find_offsets;
mod offsets;
mod schema;
mod target;

#[derive(Debug)]
pub struct CS2 {
    is_valid: bool,
    process: Process,
    offsets: Offsets,
    bvh: Arc<Mutex<HashMap<String, Bvh>>>,
    target: Target,
    players: Vec<Player>,
    previous_spectators: RefCell<HashMap<u64, u64>>,
    dead_spectators: RefCell<Vec<(String, u64, u64)>>,
    entities: Vec<Entity>,
    recoil: Recoil,
    aim: Aimbot,
    trigger: Triggerbot,
    wallhack: EspToggle,
    weapon: Weapon,
    planted_c4: Option<PlantedC4>,
}

impl Game for CS2 {
    fn is_valid(&self) -> bool {
        self.is_valid && self.process.is_valid()
    }

    fn setup(&mut self) {
        let Some(process) = Process::open(cs2::PROCESS_NAME) else {
            self.is_valid = false;
            return;
        };
        log::info!("process found, pid: {}", process.pid);
        self.process = process;

        self.offsets = match self.find_offsets() {
            Some(offsets) => offsets,
            None => {
                self.process = Process::new(-1);
                self.is_valid = false;
                return;
            }
        };
        log::info!("offsets found");

        self.is_valid = true;
    }

    fn run(&mut self, config: &Config, mouse: &mut Mouse) {
        if !self.process.is_valid() {
            self.is_valid = false;
            log::debug!("process is no longer valid");
            return;
        }

        // self.cache_players();
        self.cache_entities();

        for entity in &self.entities {
            if let Entity::Smoke(smoke) = entity {
                if config.misc.no_smoke {
                    smoke.disable(self);
                }

                if config.misc.change_smoke_color {
                    smoke.color(self, &config.misc.smoke_color);
                }
            }
        }

        self.no_flash(config);
        self.fov_changer(config);

        self.esp_toggle(config);

        self.rcs(config, mouse);
        self.triggerbot(config);

        self.triggerbot_shoot(mouse);

        self.find_target(config);

        self.aimbot(config, mouse);
    }

    fn data(&self, config: &Config, data: &mut Data) {
        data.players.clear();
        data.friendlies.clear();
        data.entities.clear();
        data.spectators.clear();
        data.spectator_names.clear();

        let mut current_spectators = HashMap::new();

        let dead_spectators = {
            let mut dead_specs = self.dead_spectators.borrow_mut();
            let specs = dead_specs.clone();
            dead_specs.clear();
            specs
        };

        let sdl_window = self.process.read::<u64>(self.offsets.direct.sdl_window);
        if sdl_window == 0 {
            data.window_position = Vec2::ZERO;
            data.window_size = Vec2::ONE;
        } else {
            data.window_position = self.process.read::<IVec2>(sdl_window + 0x18).as_vec2();
            data.window_size = self
                .process
                .read::<IVec2>(sdl_window + 0x18 + 0x08)
                .as_vec2();
        }

        let Some(local_player) = Player::local_player(self) else {
            data.weapon = Weapon::default();
            data.in_game = false;
            return;
        };
        let local_team = local_player.team(self);
        if local_team != TEAM_T && local_team != TEAM_CT {
            data.weapon = Weapon::default();
            data.in_game = false;
            return;
        }
        for player in &self.players {
            let player_data = PlayerData {
                steam_id: player.steam_id(self),
                health: player.health(self),
                armor: player.armor(self),
                position: player.position(self),
                head: player.bone_position(self, Bones::Head.u64()),
                name: player.name(self),
                weapon: player.weapon(self),
                bones: player.all_bones(self),
                has_defuser: player.has_defuser(self),
                has_helmet: player.has_helmet(self),
                has_bomb: player.has_bomb(self),
                visible: player.visible(self, &local_player),
                color: player.color(self),
                rotation: player.rotation(self),
                sound: player.is_making_sound(self),
            };
            if !self.is_ffa() && player.team(self) == local_team {
                data.friendlies.push(player_data);
            } else {
                data.players.push(player_data);
            }
        }

        data.local_player = PlayerData {
            steam_id: local_player.steam_id(self),
            health: local_player.health(self),
            armor: local_player.armor(self),
            position: local_player.position(self),
            head: local_player.bone_position(self, Bones::Head.u64()),
            name: local_player.name(self),
            weapon: local_player.weapon(self),
            bones: local_player.all_bones(self),
            has_defuser: local_player.has_defuser(self),
            has_helmet: local_player.has_helmet(self),
            has_bomb: local_player.has_bomb(self),
            visible: true,
            color: local_player.color(self),
            rotation: local_player.rotation(self),
            sound: local_player.is_making_sound(self),
        };

        data.entities = self
            .entities
            .iter()
            .map(|e| match e {
                Entity::Weapon { weapon, entity } => EntityInfo::Weapon {
                    weapon: weapon.clone(),
                    position: Player::entity(*entity).position(self),
                },
                Entity::Inferno(inferno) => EntityInfo::Inferno(inferno.info(self)),
                Entity::Smoke(smoke) => EntityInfo::Smoke(smoke.info(self)),
                Entity::Molotov(molotov) => EntityInfo::Molotov(molotov.info(self)),
                Entity::Flashbang(entity) => {
                    EntityInfo::Flashbang(GrenadeInfo::new(*entity, "Flashbang", self))
                }
                Entity::HeGrenade(entity) => {
                    EntityInfo::HeGrenade(GrenadeInfo::new(*entity, "HE Grenade", self))
                }
                Entity::Decoy(entity) => {
                    EntityInfo::Decoy(GrenadeInfo::new(*entity, "Decoy", self))
                }
            })
            .collect();

        data.weapon = local_player.weapon(self);
        data.in_game = true;
        data.is_ffa = self.is_ffa();
        data.is_custom_mode = self.is_custom_game_mode();
        data.map_name = self.current_map();
        data.aimbot_active = if self.aimbot_config(config).mode == KeyMode::Toggle {
            self.aim.active
        } else {
            false
        };
        data.triggerbot_active = if self.triggerbot_config(config).mode == KeyMode::Toggle {
            self.trigger.active
        } else {
            false
        };
        data.wallhack_active = self.esp_enabled(config);

        for (spectator_name, spectator_id, target_id) in dead_spectators {
            data.spectators.push((spectator_id, target_id));
            data.spectator_names.push((spectator_name, target_id));
            current_spectators.insert(spectator_id, target_id);
        }

        *self.previous_spectators.borrow_mut() = current_spectators;

        data.view_matrix = self.process.read::<Mat4>(self.offsets.direct.view_matrix);
        data.view_angles = local_player.view_angles(self);

        if let Some(bomb) = &self.planted_c4 {
            data.bomb.planted = bomb.is_planted(self);
            data.bomb.timer = bomb.time_to_explosion(self);
            data.bomb.position = bomb.position(self);
            data.bomb.being_defused = bomb.is_being_defused(self);
            data.bomb.defuse_remain_time = bomb.time_to_defuse(self);
        } else {
            data.bomb.planted = false;
        }
    }
}

impl CS2 {
    pub fn new(bvh: Arc<Mutex<HashMap<String, Bvh>>>) -> Self {
        Self {
            is_valid: false,
            process: Process::new(-1),
            offsets: Offsets::default(),
            bvh,
            target: Target::default(),
            players: Vec::with_capacity(64),
            previous_spectators: RefCell::new(HashMap::new()),
            dead_spectators: RefCell::new(Vec::new()),
            entities: Vec::with_capacity(128),
            recoil: Recoil::default(),
            aim: Aimbot::default(),
            trigger: Triggerbot::default(),
            wallhack: EspToggle::default(),
            weapon: Weapon::default(),
            planted_c4: None,
        }
    }

    fn aimbot_config<'a>(&self, config: &'a Config) -> &'a AimbotConfig {
        if let Some(weapon_config) = config.aim.weapons.get(&self.weapon)
            && weapon_config.aimbot.enable_override
        {
            return &weapon_config.aimbot;
        }
        &config.aim.global.aimbot
    }

    fn rcs_config<'a>(&self, config: &'a Config) -> &'a RcsConfig {
        if let Some(weapon_config) = config.aim.weapons.get(&self.weapon)
            && weapon_config.rcs.enable_override
        {
            return &weapon_config.rcs;
        }
        &config.aim.global.rcs
    }

    fn triggerbot_config<'a>(&self, config: &'a Config) -> &'a TriggerbotConfig {
        if let Some(weapon_config) = config.aim.weapons.get(&self.weapon)
            && weapon_config.triggerbot.enable_override
        {
            return &weapon_config.triggerbot;
        }
        &config.aim.global.triggerbot
    }

    fn angle_to_target(&self, local_player: &Player, position: &Vec3, aim_punch: &Vec2) -> Vec2 {
        let eye_position = local_player.eye_position(self);
        let forward = (position - eye_position).normalize();

        let mut angles = angles_from_vector(&forward) - aim_punch;
        vec2_clamp(&mut angles);

        angles
    }

    fn entity_has_owner(&self, entity: u64) -> bool {
        self.process
            .read::<i32>(entity + self.offsets.controller.owner_entity)
            != -1
    }

    // convars
    fn get_sensitivity(&self) -> f32 {
        self.process.read(self.offsets.convar.sensitivity + 0x50)
    }

    fn is_ffa(&self) -> bool {
        self.process.read::<u8>(self.offsets.convar.ffa + 0x50) == 1
    }

    fn is_custom_game_mode(&self) -> bool {
        let map = self.current_map();
        map.starts_with("workshop/")
            || map.starts_with("custom/")
            || !map.starts_with("de_") && !map.starts_with("cs_")
    }

    // misc
    pub fn is_button_down(&self, button: &KeyCode) -> bool {
        // what the actual fuck is happening here?
        let value = self.process.read::<u32>(
            self.offsets.interface.input
                + (((button.u64() >> 5) * 4) + self.offsets.direct.button_state),
        );
        ((value >> (button.u64() & 31)) & 1) != 0
    }

    fn current_time(&self) -> f32 {
        let global_vars: u64 = self.process.read(self.offsets.direct.global_vars);
        self.process.read(global_vars + 0x30)
    }

    fn current_map(&self) -> String {
        let global_vars: u64 = self.process.read(self.offsets.direct.global_vars);
        self.process
            .read_string(self.process.read(global_vars + 0x190))
    }

    fn distance_scale(&self, distance: f32) -> f32 {
        if distance > 500.0 {
            1.0
        } else {
            5.0 - (distance / 125.0)
        }
    }

    fn cache_entities(&mut self) {
        self.players.clear();
        self.entities.clear();
        self.planted_c4 = None;

        let Some(local_player) = Player::local_player(self) else {
            return;
        };

        const NUM_BUCKETS: usize = 64;
        let bucket_pointers = self
            .process
            .read_vec(self.offsets.interface.entity, 0x8 * NUM_BUCKETS);
        for bucket_index in 0..64 {
            let bucket_pointer =
                *bytemuck::from_bytes(&bucket_pointers[bucket_index * 8..(bucket_index + 1) * 8]);
            self.get_entities_in_bucket(bucket_index as u64, bucket_pointer, &local_player);
        }
    }

    fn get_entities_in_bucket(
        &mut self,
        bucket_index: u64,
        bucket_ptr: u64,
        local_player: &Player,
    ) {
        if bucket_ptr == 0 || bucket_ptr >> 48 != 0 {
            return;
        }
        const IDENTITIES_PER_BUCKET: usize = 512;
        let bucket = self.process.read_vec(
            bucket_ptr,
            IDENTITIES_PER_BUCKET * self.offsets.entity_identity.size as usize,
        );
        for index_in_bucket in 0..IDENTITIES_PER_BUCKET {
            let identity_offset = index_in_bucket * self.offsets.entity_identity.size as usize;

            let entity: u64 = *bytemuck::from_bytes(&bucket[identity_offset..identity_offset + 8]);
            if entity == 0 {
                continue;
            }

            let handle_start = identity_offset + 0x10;
            let handle: u32 = *bytemuck::from_bytes(&bucket[handle_start..handle_start + 4]);
            let handle_index = handle & 0x7FFF;
            let entity_index =
                (bucket_index as usize * IDENTITIES_PER_BUCKET + index_in_bucket) as u32;
            if entity_index != handle_index {
                continue;
            }

            let class_info_ptr: u64 =
                *bytemuck::from_bytes(&bucket[identity_offset + 8..identity_offset + 16]);
            let class_info: u64 = self.process.read(class_info_ptr + 0x30);
            let class_name_ptr: u64 = self.process.read(class_info + 0x08);
            let class = self.process.read_string(class_name_ptr);

            match class.as_str() {
                class::PLAYER_CONTROLLER => {
                    let Some(player) = Player::from_controller(entity, self) else {
                        continue;
                    };

                    if !player.is_valid(self) {
                        continue;
                    }

                    if let Some(target) = player.spectator_target(self) {
                        let spectator_id = player.steam_id(self);
                        let target_pawn = target.pawn;
                        let local_pawn = local_player.pawn;

                        if target_pawn == local_pawn {
                            let spectator_name = player.name(self);
                            let local_steam_id = local_player.steam_id(self);
                            self.dead_spectators.borrow_mut().push((
                                spectator_name,
                                spectator_id,
                                local_steam_id,
                            ));
                        }
                    }

                    if player == *local_player {
                        self.target.local_pawn_index = (handle as u64 & 0x7FFF) - 1;
                    } else {
                        self.players.push(player);
                    }
                }
                class::PLANTED_C4 => {
                    let planted_c4 = PlantedC4::new(entity);
                    if planted_c4.is_relevant(self) {
                        self.planted_c4 = Some(planted_c4)
                    }
                }
                class::INFERNO => {
                    self.entities.push(Entity::Inferno(Inferno::new(entity)));
                }
                class::SMOKE => {
                    self.entities.push(Entity::Smoke(Smoke::new(entity)));
                }
                class::MOLOTOV => self.entities.push(Entity::Molotov(Molotov::new(entity))),
                class::FLASHBANG => self.entities.push(Entity::Flashbang(entity)),
                class::HE_GRENADE => self.entities.push(Entity::HeGrenade(entity)),
                class::DECOY => self.entities.push(Entity::Decoy(entity)),
                _ => {
                    // check if weapon
                    let entity_identity: u64 = self.process.read(entity + 0x10);
                    if entity_identity == 0 {
                        continue;
                    }

                    let name_pointer = self.process.read(entity_identity + 0x20);
                    if name_pointer == 0 {
                        continue;
                    }

                    let name = self.process.read_string(name_pointer);

                    if name.starts_with("weapon_") {
                        if self.entity_has_owner(entity) {
                            continue;
                        }

                        let weapon = Weapon::from_handle(entity, self);

                        self.entities.push(Entity::Weapon { weapon, entity })
                    }
                }
            }

            // m_designerName
            /*let name_pointer: u64 =
                *bytemuck::from_bytes(&bucket[identity_offset + 0x20..identity_offset + 0x28]);
            let Some(entity) = self.entity_type(entity, name_pointer) else {
                continue;
            };
            self.entities.push(entity);*/
        }
    }
}
