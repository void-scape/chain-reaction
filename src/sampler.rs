#![allow(unused)]

use rand::Rng;
use rand::distributions::WeightedIndex;
use rand::prelude::Distribution;

pub struct Sampler<T> {
    choices: Vec<T>,
    dist: WeightedIndex<f32>,
}

impl<T> Sampler<T>
where
    T: Clone,
{
    pub fn new(choices: &[(T, f32)]) -> Self {
        Self {
            dist: WeightedIndex::new(choices.into_iter().map(|(_, weight)| *weight)).unwrap(),
            choices: choices.into_iter().map(|(t, _)| t.clone()).collect(),
        }
    }

    pub fn linear(choices: impl IntoIterator<Item = T>, start: f32, end: f32) -> Self {
        let choices = choices.into_iter().collect::<Vec<_>>();

        assert!(end > start);
        assert!(choices.len() > 1);
        assert!(start.is_sign_positive() && end.is_sign_positive());

        let len = choices.len();
        let step = (end - start) / (len - 1) as f32;
        let dist = WeightedIndex::new((0..len).map(|i| step * i as f32)).unwrap();

        Self { choices, dist }
    }

    pub fn sample(&self, rng: &mut impl Rng) -> T {
        self.choices[self.dist.sample(rng)].clone()
    }

    pub fn iter(&self, rng: &mut impl Rng, samples: usize) -> impl Iterator<Item = T> {
        (0..samples).map(|_| self.sample(rng))
    }
}
