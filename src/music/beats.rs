use bevy::prelude::*;
use bevy_seedling::{pool::Sampler, prelude::*};

/// A measure of where the music is in its beat pattern,
/// normalized for each piece of music.
#[derive(Component)]
pub struct MusicBeats {
    position: f32,
    bpm: f32,
}

impl MusicBeats {
    pub fn get(&self) -> f32 {
        self.position
    }

    pub fn new(bpm: f32) -> Self {
        Self {
            position: 0f32,
            bpm,
        }
    }
}

pub fn update_beats(
    mut beats: Query<(&mut MusicBeats, &Sampler)>,
    mut context: ResMut<AudioContext>,
) {
    let sample_rate = context
        .with(|c| c.stream_info().map(|i| i.sample_rate.get()))
        .unwrap_or(441000);

    for (mut beats, sampler) in beats.iter_mut() {
        let beats_per_second = beats.bpm / 60.0;
        let Some(frames) = sampler.try_playhead_frames() else {
            continue;
        };
        let seconds = frames as f32 / sample_rate as f32;

        beats.position = seconds * beats_per_second;
    }
}
