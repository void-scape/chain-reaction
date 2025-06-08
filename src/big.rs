use dashu::integer::IBig;
use dashu::integer::ops::EstimatedLog2;

#[derive(Debug, Default, Clone, serde::Serialize, serde::Deserialize)]
#[serde(transparent)]
pub struct BigPoints(pub IBig);

impl BigPoints {
    pub fn new(score: i32) -> Self {
        Self(IBig::from(score))
    }
}

const EXP_THRESHOLD: IBig = dashu::ibig!(1_000_000_000);

impl core::fmt::Display for BigPoints {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let value = self.0.to_string();

        if self.0 >= EXP_THRESHOLD {
            let base_2 = self.0.log2_est();
            let base_10 = (base_2 * 0.301).round();

            let first = value.chars().next().unwrap_or('1');
            let next_four = value.chars().skip(1).take(4).collect::<String>();
            let next_four = next_four.trim_end_matches('0');
            let next_four = if next_four.is_empty() { "0" } else { next_four };

            write!(f, "{first}.{next_four}e{base_10}")
        } else {
            write!(f, "{value}")
        }
    }
}
