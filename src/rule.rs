use bevy::{
    prelude::*,
    render::extract_resource::{ExtractResource, ExtractResourcePlugin},
};
use bytemuck::{Pod, Zeroable};

use crate::rtmaterial::RTVolumeMaterial;

#[derive(Clone, Copy, Default, PartialEq, Eq, Debug)]
pub struct Value([bool; 27]);

impl Value {
    pub fn at(&self, index: usize) -> &bool {
        &self.0[index]
    }

    pub fn at_mut(&mut self, index: usize) -> &mut bool {
        &mut self.0[index]
    }

    pub fn try_parse(s: &str) -> Option<Value> {
        if s.len() == 0 {
            return Some(Value::default());
        }
        let mut res = Value::default();
        for value in s.split(",") {
            let value = value.trim();
            if let Some((r0, r1)) = value.split_once('-') {
                let r0 = r0.trim_end();
                let r1 = r1.trim_start();
                for i in r0.parse::<usize>().ok()?..=r1.parse::<usize>().ok()? {
                    *res.0.get_mut(26 - i)? = true;
                }
            } else {
                *res.0.get_mut(26 - value.parse::<usize>().ok()?)? = true;
            }
        }
        Some(res)
    }
}

impl ToString for Value {
    fn to_string(&self) -> String {
        let mut elems = Vec::new();
        let mut i = 0;
        while i < 27 {
            if *self.at(i) {
                let end = 26 - i;
                while i < 27 && *self.at(i) {
                    i += 1;
                }
                let start = 26 - (i - 1);
                if start == end {
                    elems.push(start.to_string());
                } else {
                    elems.push(format!("{}-{}", start, end));
                }
            } else {
                i += 1;
            }
        }
        if elems.is_empty() {
            String::new()
        } else {
            elems
                .iter()
                //.rev()
                .skip(1)
                .fold(elems[0].clone(), |acc, x| format!("{},{}", x, acc))
        }
    }
}

impl From<Vec<usize>> for Value {
    fn from(vec: Vec<usize>) -> Self {
        let mut value = [false; 27];
        for i in vec {
            value[26 - i] = true;
        }
        Self(value)
    }
}

impl From<Value> for u32 {
    fn from(v: Value) -> Self {
        v.0.iter().fold(0, |acc, &b| (acc << 1) | (b as u32))
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum NeighborMode {
    Moore = 0,
    VonNeumann = 1,
}

impl NeighborMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Moore => "Moore",
            Self::VonNeumann => "VonNeumann",
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum ColorMode {
    Single(Color),
    StateLerp(Color, Color),
    DistToCenter(Color, Color),
    Neighbour(Color, Color),
}

impl Eq for ColorMode {}

impl ColorMode {
    pub fn kind(&self) -> ColorModeKind {
        match self {
            Self::Single(_) => ColorModeKind::Single,
            Self::StateLerp(_, _) => ColorModeKind::StateLerp,
            Self::DistToCenter(_, _) => ColorModeKind::DistToCenter,
            Self::Neighbour(_, _) => ColorModeKind::Neighbour,
        }
    }

    pub fn colors(&self) -> (Color, Color) {
        match self {
            Self::Single(c) => (*c, *c),
            Self::StateLerp(c1, c2) => (*c1, *c2),
            Self::DistToCenter(c1, c2) => (*c1, *c2),
            Self::Neighbour(c1, c2) => (*c1, *c2),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ColorModeKind {
    Single,
    StateLerp,
    DistToCenter,
    Neighbour,
}

impl ColorModeKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Single => "Single",
            Self::StateLerp => "State Lerp",
            Self::DistToCenter => "Distance To Center",
            Self::Neighbour => "Neighbour Count Lerp",
        }
    }

    pub fn update(&self, mode: &mut ColorMode) {
        let colors = mode.colors();
        match self {
            Self::Single => {
                *mode = ColorMode::Single(colors.0);
            }
            Self::StateLerp => {
                *mode = ColorMode::StateLerp(colors.0, colors.1);
            }
            Self::DistToCenter => {
                *mode = ColorMode::DistToCenter(colors.0, colors.1);
            }
            Self::Neighbour => {
                *mode = ColorMode::Neighbour(colors.0, colors.1);
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum SpawnMode {
    Random(f32),
    MengerSponge,
}

impl Eq for SpawnMode {}

impl SpawnMode {
    pub fn kind(&self) -> SpawnModeKind {
        match self {
            Self::Random(_) => SpawnModeKind::Random,
            Self::MengerSponge => SpawnModeKind::MengerSponge,
        }
    }

    pub fn float(&self) -> f32 {
        match self {
            Self::Random(f) => *f,
            Self::MengerSponge => 0.0,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum SpawnModeKind {
    Random,
    MengerSponge,
}

impl SpawnModeKind {
    pub fn update(&self, mode: &mut SpawnMode) {
        let float = mode.float();
        match self {
            Self::Random => {
                *mode = SpawnMode::Random(float);
            }
            Self::MengerSponge => {
                *mode = SpawnMode::MengerSponge;
            }
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Random => "Random",
            Self::MengerSponge => "Menger Sponge",
        }
    }
}
#[repr(C)]
#[derive(Clone, Copy, PartialEq, Default, Pod, Zeroable)]
pub struct GPURule {
    size: u32,
    spawn_mode: u32,
    spawn_chance: f32,
    survival: u32,
    birth: u32,
    states: u32,
    neighbor_mode: u32,
    color_mode: u32,
    color0: [f32; 4],
    color1: [f32; 4],
}

impl From<&Rule> for GPURule {
    fn from(rule: &Rule) -> Self {
        let (color_mode, color0, color1) = match rule.color_mode {
            ColorMode::Single(c) => (0, c.as_rgba_f32(), [0.0; 4]),
            ColorMode::StateLerp(c0, c1) => (1, c0.as_rgba_f32(), c1.as_rgba_f32()),
            ColorMode::DistToCenter(c0, c1) => (2, c0.as_rgba_f32(), c1.as_rgba_f32()),
            ColorMode::Neighbour(c0, c1) => (3, c0.as_rgba_f32(), c1.as_rgba_f32()),
        };
        Self {
            size: rule.size,
            spawn_mode: rule.spawn_mode.kind() as u32,
            spawn_chance: rule.spawn_mode.float(),
            survival: rule.survival.into(),
            birth: rule.birth.into(),
            states: rule.states,
            neighbor_mode: rule.neighbor_mode as u32,
            color_mode,
            color0,
            color1,
        }
    }
}

#[derive(Clone, Resource, PartialEq, Eq, Debug, ExtractResource)]
pub struct Rule {
    pub size: u32,
    pub spawn_mode: SpawnMode,
    pub survival: Value,
    pub birth: Value,
    pub states: u32,
    pub neighbor_mode: NeighborMode,
    pub color_mode: ColorMode,
}

fn update_materials(
    rule: Res<Rule>,
    material_query: Query<&Handle<RTVolumeMaterial>>,
    mut materials: ResMut<Assets<RTVolumeMaterial>>,
) {
    for material in material_query.iter() {
        if let Some(material) = materials.get_mut(material) {
            if material.rule != *rule {
                material.rule = rule.clone();
            }
        }
    }
}

pub struct RulePlugin;

impl Plugin for RulePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Rule {
            size: 255,
            spawn_mode: SpawnMode::MengerSponge,
            survival: vec![4].into(),
            birth: vec![4, 5, 6, 7].into(),
            states: 5,
            neighbor_mode: NeighborMode::Moore,
            color_mode: ColorMode::StateLerp(Color::rgb_u8(176, 0, 188), Color::rgb_u8(99, 0, 104)),
        })
        .add_plugin(ExtractResourcePlugin::<Rule>::default())
        .add_system(update_materials);
    }
}
