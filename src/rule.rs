use bevy::{
    prelude::*,
    render::{
        render_resource::{Buffer, BufferInitDescriptor, BufferUsages},
        renderer::{RenderDevice, RenderQueue},
        RenderApp, RenderStage,
    },
};
use bytemuck::{Pod, Zeroable};

#[derive(Clone, Copy, Default)]
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
            if let Some((r0, r1)) = value.split_once("..=") {
                let r0 = r0.trim_end();
                let r1 = r1.trim_start();
                for i in r0.parse::<usize>().ok()?..=r1.parse::<usize>().ok()? {
                    *res.0.get_mut(26 - i)? = true;
                }
            } else if let Some((r0, r1)) = value.split_once("..") {
                let r0 = r0.trim_end();
                let r1 = r1.trim_start();
                match (r0.len(), r1.len()) {
                    (0, 0) => {
                        res.0 = [true; 27];
                    }
                    (0, _) => {
                        for i in 0..r1.parse::<usize>().ok()? {
                            *res.0.get_mut(26 - i)? = true;
                        }
                    }
                    (_, 0) => {
                        for i in r0.parse::<usize>().ok()?..27 {
                            *res.0.get_mut(26 - i)? = true;
                        }
                    }
                    _ => {
                        for i in r0.parse::<usize>().ok()?..r1.parse::<usize>().ok()? {
                            *res.0.get_mut(26 - i)? = true;
                        }
                    }
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
                    elems.push(format!("{}..={}", start, end));
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

#[derive(Clone, Copy, PartialEq, Eq)]
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

#[derive(Clone)]
pub enum ColorMode {
    Single(Color),
    StateLerp(Color, Color),
    DistToCenter(Color, Color),
    Neighbour(Color, Color),
}

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

#[repr(C)]
#[derive(Clone, Copy, PartialEq, Default, Pod, Zeroable)]
struct GPURule {
    pub size: u32,
    spawn_chance: f32,
    survival: u32,
    birth: u32,
    states: u32,
    neighbor_mode: u32,
    color_mode: u32,
    padding: [f32; 1],
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
            spawn_chance: rule.spawn_chance,
            survival: rule.survival.into(),
            birth: rule.birth.into(),
            states: rule.states,
            neighbor_mode: rule.neighbor_mode as u32,
            color_mode,
            color0,
            color1,
            padding: [0.0; 1],
        }
    }
}

#[derive(Clone)]
pub struct Rule {
    pub size: u32,
    pub spawn_chance: f32,
    pub survival: Value,
    pub birth: Value,
    pub states: u32,
    pub neighbor_mode: NeighborMode,
    pub color_mode: ColorMode,
}

impl Rule {}
#[derive(Clone, Default)]
pub struct RuleBuffer(pub Option<Buffer>);

fn extract_rule(mut commands: Commands, rule: Res<Rule>) {
    commands.insert_resource(rule.clone());
}

fn queue_create_buffer(
    rule: Res<Rule>,
    mut rbuffer: ResMut<RuleBuffer>,
    render_device: Res<RenderDevice>,
    queue: ResMut<RenderQueue>,
    mut last_rule: Local<GPURule>,
) {
    if let Some(buffer) = &mut rbuffer.0 {
        let rule = GPURule::from(&*rule);
        if !last_rule.eq(&rule) {
            queue.write_buffer(buffer, 0, bytemuck::bytes_of(&rule));
            *last_rule = rule;
        }
    } else {
        *last_rule = GPURule::from(&*rule);
        let buffer = render_device.create_buffer_with_data(&BufferInitDescriptor {
            label: None,
            contents: bytemuck::bytes_of(&*last_rule),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });
        rbuffer.0 = Some(buffer);
    }
}

pub struct RulePlugin;

impl Plugin for RulePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Rule {
            size: 256,
            spawn_chance: 0.99,
            survival: vec![4].into(),
            birth: vec![4, 5, 6, 7].into(),
            states: 5,
            neighbor_mode: NeighborMode::Moore,
            color_mode: ColorMode::StateLerp(Color::rgb_u8(176, 0, 188), Color::rgb_u8(99, 0, 104)),
        });
        let render_app = app.sub_app_mut(RenderApp);

        render_app
            .init_resource::<RuleBuffer>()
            .add_system_to_stage(RenderStage::Extract, extract_rule)
            .add_system_to_stage(RenderStage::Queue, queue_create_buffer.label("rule_buffer"));
    }
}