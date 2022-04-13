use bevy::prelude::*;
use bevy_egui::{
    egui::{self, panel::Side},
    EguiContext, EguiPlugin,
};

use crate::{
    ca_compute::{ReInit, UpdateTime},
    fly_cam::MovementSettings,
    rule::{ColorMode, ColorModeKind, NeighborMode, Rule, SpawnMode, SpawnModeKind, Value},
    Meshes, START_SENSITIVITY, START_SPEED,
};

pub struct GuiPlugin;

impl Plugin for GuiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(EguiPlugin).add_system(egui_system);
    }
}

#[derive(Default)]
struct State {
    survival: String,
    birth: String,
    size: u32,
    fps: f32,
}

fn egui_system(
    mut ctx: ResMut<EguiContext>,
    rule: Option<ResMut<Rule>>,
    update_time: Option<ResMut<UpdateTime>>,
    reinit: Option<ResMut<ReInit>>,
    movement: Option<ResMut<MovementSettings>>,
    meshes: Option<ResMut<Meshes>>,
    time: Res<Time>,
    mut state: Local<State>,
) {
    egui::SidePanel::new(Side::Left, "settings").show(ctx.ctx_mut(), |ui| {
        if let Some(mut rule) = rule {
            if let Some(mut reinit) = reinit {
                ui.heading("Spawn");
                let mut mode = rule.spawn_mode.kind();
                egui::ComboBox::from_label("Spawn mode")
                    .selected_text(mode.as_str())
                    .show_ui(ui, |ui| {
                        ui.selectable_value(
                            &mut mode,
                            SpawnModeKind::Random,
                            SpawnModeKind::Random.as_str(),
                        );
                        ui.selectable_value(
                            &mut mode,
                            SpawnModeKind::MengerSponge,
                            SpawnModeKind::MengerSponge.as_str(),
                        );
                    });
                mode.update(&mut rule.spawn_mode);
                match &mut rule.spawn_mode {
                    SpawnMode::Random(f) => {
                        *f = 1.0 - *f;
                        ui.add(egui::Slider::new(f, 0.0..=1.0));
                        *f = 1.0 - *f;
                    }
                    _ => {}
                }
                ui.end_row();
                if ui.button("Reset").clicked() {
                    reinit.0 = true;
                }
            }

            ui.heading("Rule");
            ui.end_row();

            ui.label("Size");
            if state.size == 0 {
                state.size = rule.size;
            }
            let res = ui.add(egui::Slider::new(&mut state.size, 2..=1024));
            if res.lost_focus() || res.drag_released() {
                rule.size = state.size;
            }
            ui.end_row();

            ui.label("Survival");
            let survival = ui.text_edit_singleline(&mut state.survival);
            if survival.changed() {
                if let Some(survival) = Value::try_parse(&state.survival) {
                    rule.survival = survival;
                }
            } else if !survival.has_focus() {
                state.survival = rule.survival.to_string();
            }

            ui.label("Birth");
            let birth = ui.text_edit_singleline(&mut state.birth);
            if birth.changed() {
                if let Some(birth) = Value::try_parse(&state.birth) {
                    rule.birth = birth;
                }
            } else if !birth.has_focus() {
                state.birth = rule.birth.to_string();
            }

            ui.label("States");
            ui.add(egui::Slider::new(&mut rule.states, 1..=40));
            ui.end_row();

            egui::ComboBox::from_label("Neighbor mode")
                .selected_text(rule.neighbor_mode.as_str())
                .show_ui(ui, |ui| {
                    ui.selectable_value(
                        &mut rule.neighbor_mode,
                        NeighborMode::Moore,
                        NeighborMode::Moore.as_str(),
                    );
                    ui.selectable_value(
                        &mut rule.neighbor_mode,
                        NeighborMode::VonNeumann,
                        NeighborMode::VonNeumann.as_str(),
                    );
                });
            let mut mode = rule.color_mode.kind();
            egui::ComboBox::from_label("Color mode")
                .selected_text(mode.as_str())
                .show_ui(ui, |ui| {
                    ui.selectable_value(
                        &mut mode,
                        ColorModeKind::Single,
                        ColorModeKind::Single.as_str(),
                    );
                    ui.selectable_value(
                        &mut mode,
                        ColorModeKind::StateLerp,
                        ColorModeKind::StateLerp.as_str(),
                    );
                    ui.selectable_value(
                        &mut mode,
                        ColorModeKind::DistToCenter,
                        ColorModeKind::DistToCenter.as_str(),
                    );
                    ui.selectable_value(
                        &mut mode,
                        ColorModeKind::Neighbour,
                        ColorModeKind::Neighbour.as_str(),
                    );
                });
            mode.update(&mut rule.color_mode);
            fn color_edit(ui: &mut egui::Ui, color: &mut Color) -> egui::Response {
                match *color {
                    Color::Rgba {
                        red,
                        green,
                        blue,
                        alpha,
                    } => {
                        let mut rgb = [red, green, blue];
                        let res = ui.color_edit_button_rgb(&mut rgb);
                        *color = Color::Rgba {
                            red: rgb[0],
                            green: rgb[1],
                            blue: rgb[2],
                            alpha,
                        };
                        res
                    }
                    Color::Hsla {
                        hue,
                        saturation,
                        lightness,
                        alpha,
                    } => {
                        let mut c = egui::color::Hsva::new(hue, saturation, lightness, alpha);
                        let res = ui.color_edit_button_hsva(&mut c);
                        *color = Color::Hsla {
                            hue: c.h,
                            saturation: c.s,
                            lightness: c.v,
                            alpha: c.a,
                        };
                        res
                    }
                    _ => todo!(),
                }
            }
            match &mut rule.color_mode {
                ColorMode::Single(c) => {
                    ui.label("Color");
                    color_edit(ui, c);
                    ui.end_row();
                }

                ColorMode::StateLerp(c0, c1)
                | ColorMode::DistToCenter(c0, c1)
                | ColorMode::Neighbour(c0, c1) => {
                    ui.label("Color A");
                    color_edit(ui, c0);
                    ui.end_row();

                    ui.label("Color B");
                    color_edit(ui, c1);
                    ui.end_row();
                }
            }
        }

        ui.heading("Misc");
        if let Some(mut update_time) = update_time {
            ui.label("Update time");
            ui.add(egui::Slider::new(&mut update_time.0, 0.0..=5.0).logarithmic(true));
            if update_time.0 == 5.0 {
                update_time.0 = f64::INFINITY;
            }
            ui.end_row();
        }
        if let Some(mut meshes) = meshes {
            egui::ComboBox::from_label("Shape")
                .selected_text(meshes.meshes[meshes.current].0)
                .show_ui(ui, |ui| {
                    for i in 0..meshes.meshes.len() {
                        let s = meshes.meshes[i].0;
                        ui.selectable_value(&mut meshes.current, i, s);
                    }
                });
        }
        if let Some(mut movement) = movement {
            ui.heading("Movement");
            ui.label("Speed");
            let mut speed = movement.speed / START_SPEED;
            ui.add(egui::Slider::new(&mut speed, 0.1..=10.0).logarithmic(true));
            movement.speed = speed * START_SPEED;
            ui.end_row();
            ui.label("Sensitivity");
            let mut sensitivity = movement.sensitivity / START_SENSITIVITY;
            ui.add(egui::Slider::new(&mut sensitivity, 0.1..=10.0).logarithmic(true));
            movement.sensitivity = sensitivity * START_SENSITIVITY;
            ui.end_row();
        }

        ui.heading("Info");
        const SMOOTHING: f32 = 0.9;
        if time.delta_seconds() > 0.0 {
            state.fps = state.fps * SMOOTHING + (1.0 - SMOOTHING) * 1.0 / time.delta_seconds();
        }
        ui.label(format!("FPS: {}", state.fps as u32));
    });
}
