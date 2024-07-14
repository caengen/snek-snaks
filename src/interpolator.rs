use bevy::prelude::*;
use bevy_tween::prelude::*;

pub use bevy_tween::interpolate::*;

pub fn custom_interpolators_plugin(app: &mut App) {
    app.add_tween_systems(bevy_tween::component_tween_system::<AtlasIndex>());
}

pub fn atlas_index(start: usize, end: usize) -> AtlasIndex {
    AtlasIndex { start, end }
}

pub struct AtlasIndex {
    pub start: usize,
    pub end: usize,
}

impl Interpolator for AtlasIndex {
    type Item = TextureAtlas;

    fn interpolate(&self, item: &mut Self::Item, value: f32) {
        let start = self.start as f32;
        let end = self.end as f32;
        item.index = start.lerp(end, value).floor() as usize;
    }
}
