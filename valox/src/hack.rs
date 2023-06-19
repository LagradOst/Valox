use std::{
    collections::{HashMap, HashSet},
    ops::Sub,
    time::Instant,
};

use crate::{bones::CommonBones, *};
use ue4_rs::prelude::egui::{Align2, Color32, FontId, Pos2};
use winapi::um::winuser::{GetAsyncKeyState, VK_RMENU};

struct HackData {
    // fps
    last_frame: Instant,

    // settings
    tracers_enabled: bool,
    box2d_enabled: bool,
    bones_enabled: bool,
    hp_enabled: bool,
    tracers_color: Color32,
    regular_color: Color32,
    visible_color: Color32,
    bomb_enabled: bool,
    dropped_items_enabled: bool,
    ice_enabled: bool,

    // smart aactor looping variables, stores the static varaibles when the object is created
    old_actors: HashSet<AActorPtr>,
    bots: HashSet<ATrainingBot_PC_CPtr>,
    ice_walls: HashMap<AGameObject_Thorne_E_Wall_Segment_Fortifying_CPtr, FVector>,
    dropped_items: HashMap<AEquippableGroundPickup_CPtr, (FVector, &'static str)>,
    bomb: HashMap<ATimedBomb_CPtr, (FVector, f32)>,
}

impl HackData {
    fn draw_player(&self, camera: &EguiCamera, character: AShooterCharacterPtr) -> ReadResult<()> {
        let mesh = character.mesh()?;
        if character.b_locally_hidden()? {
            return Ok(());
        }

        let c2w = mesh.component_to_world()?;

        // get hp via damage_sections
        let hp_values = character
            .damage_handler()?
            .damage_sections()?
            .into_iter()
            .try_fold(HpTypes::default(), |mut acc, dmg| {
                if dmg.is_invalid() {
                    return Ok(acc);
                }
                let hp = HpValue {
                    value: dmg.life()?,
                    max: dmg.maximum_life()?,
                };
                match dmg.damage_type()? {
                    DamageSectionType::Health => {
                        acc.hp = hp;
                    }
                    DamageSectionType::Shield => {
                        acc.sheild = hp;
                    }
                    DamageSectionType::Temporary => {
                        acc.temp = hp;
                    }
                }
                Ok(acc)
            })?;

        if hp_values.hp.value <= 0. {
            return Ok(());
        }

        if self.tracers_enabled {
            camera.draw_tracers(c2w.translation, self.tracers_color);
        }
        
        // simple culling
        if camera.forwards.dot((camera.world_location - c2w.translation).normalized()) > 0. {
            return Ok(());
        }
        
        let color = if mesh.b_recently_rendered()? {
            self.visible_color
        } else {
            self.regular_color
        };

        if self.box2d_enabled || self.bones_enabled {
            let bones = mesh.bone_array()?.bones();
            let skeleton = match bones.len() {
                104 => &bones::MALE_BONES,
                101 => &bones::FEMALE_BONES,
                103 => &bones::BOT_BONES,
                _ => Err(MemoryError::BadData)?,
            };

            if self.box2d_enabled {
                let root = c2w.get_bone_with_rotation(&bones[CommonBones::Root as usize]);
                let head = c2w.get_bone_with_rotation(&bones[CommonBones::Head as usize]);
                let head_screen = camera.w2s(head).to_vec2();
                let root_screen = camera.w2s(root).to_vec2();
                const WIDTH: FReal = 80.;
                let width = camera
                    .w2s(head + camera.right * WIDTH)
                    .x
                    .sub(head_screen.x)
                    .abs();
                let height = (root_screen.y - head_screen.y) * 1.2;
                camera.draw_2d_box(
                    ((head_screen + root_screen) * 0.5).to_pos2(),
                    width,
                    height,
                    Color32::from_black_alpha(40),
                    Stroke::new(1., color),
                );
            }

            if self.bones_enabled {
                camera.draw_skeleton(
                    c2w,
                    &bones,
                    skeleton,
                    SkeletonRenderMode::Bezier {
                        stroke: Stroke::new(1., color),
                        radius: 10000.0,
                    },
                );
            }
        }

        if self.hp_enabled {
            if let Some(screen) = camera.w2sc(c2w.translation) {
                camera.draw_outline_text(
                    screen,
                    Align2::CENTER_TOP,
                    format!(
                        "{}hp",
                        hp_values.hp.value + hp_values.sheild.value + hp_values.temp.value
                    ),
                    FontId::proportional(13.),
                )
            }
        }

        Ok(())
    }

    fn main_loop(&mut self, layer: &RenderLayer) -> ReadResult<()> {
        let gworld = gworld()?;

        let controller = gworld
            .owning_game_instance()?
            .local_players()?
            .index(0)?
            .player_controller()?;

        let local_pawn = controller
            .acknowledged_pawn()?
            .cast::<AShooterCharacterPtr>();

        let minimal_info = controller
            .player_camera_manager()?
            .camera_cache_private()?
            .pov;

        let local_team = local_pawn
            .player_state()?
            .cast::<AAresPlayerStateBasePtr>()
            .team_component()?
            .team_id()?;

        let camera = EguiCamera::from_minimal(minimal_info, layer);
        let game_state = gworld.game_state()?.cast::<AGameStateBasePtr>();
        let player_array = game_state.player_array()?.cast::<AShooterPlayerStatePtr>();

        for player in player_array {
            let _: ReadResult<()> = try {
                let character = player.spawned_character()?;
                if character == local_pawn || player.team_component()?.team_id()? == local_team {
                    continue;
                }
                self.draw_player(&camera, character)?;
            };
        }

        self.update_misc_data(gworld.persistent_level()?.actors()?.to_vec());

        self.draw_bots(&camera);

        if self.bomb_enabled {
            self.draw_bomb(&camera);
        }

        if self.dropped_items_enabled {
            self.draw_dopped_items(&camera);
        }

        if self.ice_enabled {
            self.draw_ice_wall(&camera);
        }

        Ok(())
    }

    // update misc using https://www.unknowncheats.me/forum/valorant/585072-actors-looping.html
    // this takes less aprox then 1ms to do, so it is very cheap, worst case ~25ms when all actors refresh aka joining a match
    fn update_misc_data(&mut self, actors: Vec<AActorPtr>) {
        let new_actors: HashSet<AActorPtr> = HashSet::from_iter(actors);

        let added_actors = new_actors.difference(&self.old_actors);
        let removed_actors = self.old_actors.difference(&new_actors);

        for &actor in added_actors {
            let _: ReadResult<()> = try {
                if let Ok(bot) = actor.try_cast::<ATrainingBot_PC_CPtr>() {
                    self.bots.insert(bot);
                } else if let Ok(ice_wall) =
                    actor.try_cast::<AGameObject_Thorne_E_Wall_Segment_Fortifying_CPtr>()
                {
                    self.ice_walls.insert(
                        ice_wall,
                        ice_wall.root_component()?.component_to_world()?.translation,
                    );
                } else if let Ok(dropped_item) = actor.try_cast::<AEquippableGroundPickup_CPtr>() {
                    self.dropped_items.insert(
                        dropped_item,
                        (
                            dropped_item
                                .root_component()?
                                .component_to_world()?
                                .translation,
                            "None",
                        ),
                    );
                } else if let Ok(bomb) = actor.try_cast::<ATimedBomb_CPtr>() {
                    self.bomb.insert(
                        bomb,
                        (
                            bomb.root_component()?.component_to_world()?.translation,
                            bomb.bomb_explode_outer_radius()?,
                        ),
                    );
                }
            };
        }

        for &actor in removed_actors {
            self.bots.remove(&actor.cast());
            self.ice_walls.remove(&actor.cast());
            self.bomb.remove(&actor.cast());
            self.bots.remove(&actor.cast());
            self.dropped_items.remove(&actor.cast());
        }
        self.old_actors = new_actors;
    }

    fn draw_bots(&self, camera: &EguiCamera) {
        for &bot in &self.bots {
            let _: ReadResult<()> = try {
                // we assume that all bots are enemies
                self.draw_player(&camera, bot.cast())?;
            };
        }
    }

    fn draw_bomb(&self, camera: &EguiCamera) {
        for (ptr, &(location, radius)) in &self.bomb {
            let _: ReadResult<()> = try {
                const MAX_PROGRESS: f32 = 6.984602;

                if ptr.bomb_has_exploded()? {
                    continue;
                }

                let progress = ptr.defuse_progress()? / MAX_PROGRESS;
                let time = ptr.time_remaining_to_explode()?;
                let distance_to_safe = radius - camera.distance(location) + 400.; // some margin of error
                if 0. < distance_to_safe {
                    camera.draw_outline_text_color(
                        Pos2::new(camera.screen_center.x, 50.),
                        Align2::CENTER_CENTER,
                        format!("Spike {:.0}m {time:.01}s", distance_to_safe * 0.01),
                        FontId::monospace(15.),
                        Color32::RED,
                    );
                } else {
                    camera.draw_outline_text_color(
                        Pos2::new(camera.screen_center.x, 50.),
                        Align2::CENTER_CENTER,
                        format!("Spike {time:.01}s"),
                        FontId::monospace(15.),
                        Color32::RED,
                    );
                }
                let Some(screen) = camera.w2sc(location) else {
                    continue;
                };
                camera.draw_outline_text(
                    screen,
                    Align2::CENTER_CENTER,
                    format!(
                        "{} {time:.01}s {:.0}%",
                        if ptr.bomb_defuse_state()? {
                            "Defusing"
                        } else {
                            "Spike"
                        },
                        progress * 100.0
                    ),
                    FontId::monospace(12.),
                );
            };
        }
    }

    fn draw_dopped_items(&mut self, camera: &EguiCamera) {
        for (dropped_item, (location, name)) in self.dropped_items.iter_mut() {
            // bro idk why, but I have to lateinit because AAresOnGroundEquippable is not valid on the spawnframe nor is displayname
            if *name == "None" {
                let _: ReadResult<()> = try {
                    *name = GUN_PAIRS
                        .get(
                            &dropped_item
                                .cast::<AAresOnGroundEquippablePtr>()
                                .my_equippable()?
                                .uobject()?
                                .read_name()?,
                        )
                        .unwrap_or(&"Unknown");
                };
            }

            let Some(screen) = camera.w2sc(*location) else {
                continue;
            };
            camera.draw_outline_text(
                screen,
                Align2::CENTER_CENTER,
                format!("{name}"),
                FontId::monospace(12.),
            );
        }
    }

    fn draw_ice_wall(&self, camera: &EguiCamera) {
        for (ptr, &location) in &self.ice_walls {
            let _: ReadResult<()> = try {
                let hp = ptr.damage_handler()?.damage_sections()?.index(0)?.life()?;
                if hp <= 0.0 {
                    continue;
                }
                let Some(screen) = camera.w2sc(location) else {
                    continue;
                };
                camera.draw_outline_text(
                    screen,
                    Align2::CENTER_CENTER,
                    format!("{hp:0}"),
                    FontId::monospace(12.),
                );
            };
        }
    }

    pub fn show_ui(&mut self, ctx: &egui::Context) {
        egui::Window::new("Settings").show(ctx, |ui| {
            ui.checkbox(&mut self.bones_enabled, "Bones");
            ui.checkbox(&mut self.box2d_enabled, "2D Box");
            ui.checkbox(&mut self.hp_enabled, "Hp");
            ui.checkbox(&mut self.tracers_enabled, "Tracers");
            ui.checkbox(&mut self.ice_enabled, "Ice Walls");
            ui.checkbox(&mut self.bomb_enabled, "Bomb");
            ui.checkbox(&mut self.dropped_items_enabled, "Dropped Items");

            ui.label("Visible color");
            ui.color_edit_button_srgba(&mut self.visible_color);
            ui.label("Regular color");
            ui.color_edit_button_srgba(&mut self.regular_color);
            ui.label("Tracer color");
            ui.color_edit_button_srgba(&mut self.tracers_color);
        });
    }
}

impl Draw for HackData {
    fn run(&mut self, show_ui: &mut bool, ctx: &egui::Context) {
        let render_layer = RenderLayer::new(&ctx);
        let start = Instant::now();
        _ = self.main_loop(&render_layer);
        let end = Instant::now();

        unsafe {
            if GetAsyncKeyState(VK_RMENU) & 1 != 0 {
                *show_ui = !*show_ui;
            }
        }

        if *show_ui {
            self.show_ui(ctx);
        }

        // ups is how long the core logic took, this is what you should focus on if you want to improve fps
        // fps is logic + render, render times cant be optimized
        render_layer.painter.text(
            Pos2::new(10., 10.),
            Align2::LEFT_TOP,
            format!(
                "{:.0}ups\n{:.0}fps",
                end.duration_since(start).as_secs_f32().recip(),
                Instant::now()
                    .duration_since(self.last_frame)
                    .as_secs_f32()
                    .recip()
            ),
            FontId::proportional(12.0),
            Color32::WHITE,
        );
        self.last_frame = Instant::now();
    }

    fn setup(&mut self, _ctx: &egui::Context) {
        // change style ect
    }

    fn save(&self) {
        // save you settings here
    }
}

pub fn start_hack() -> ReadResult<()> {
    initalize()?;
    let hack = HackData {
        last_frame: Instant::now(),
        old_actors: Default::default(),
        bots: Default::default(),
        ice_walls: Default::default(),
        dropped_items: Default::default(),
        bomb: Default::default(),
        regular_color: Color32::WHITE,
        visible_color: Color32::RED,
        tracers_color: Color32::WHITE,
        tracers_enabled: true,
        box2d_enabled: true,
        bones_enabled: true,
        hp_enabled: true,
        bomb_enabled: true,
        dropped_items_enabled: true,
        ice_enabled: true,
    };

    start_overlay(Box::new(hack), "BlueFire1337".to_owned(), false);
    Ok(())
}
