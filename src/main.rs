mod math;

use math::binomial_distribution;
use rand::{Rng, rngs::ThreadRng};

struct Team;

impl Team {
    fn probability_to_win_against(&self, other: &Self) -> f32 {
        0.5
    }
}

trait Component<const N: usize> {
    fn run(&self, input: &mut [Team; N], rng: &mut ThreadRng);
}

struct BestOfN {
    n: usize,
}

impl BestOfN {
    fn new(n: usize) -> Self {
        assert!(n.rem_euclid(2) == 1);
        Self {
            n,
        }
    }

}

impl Component<2> for BestOfN {
    fn run(&self, input: &mut [Team; 2], rng: &mut ThreadRng) {
        let p = input[0].probability_to_win_against(&input[1]);
        let total: f32 = (0..(self.n-1)/2).map(|k| binomial_distribution(p, self.n, k)).sum();
        if rng.gen_range(0.0..=1.0) > total {
            input.swap(0, 1);
        }
    }
}

fn main() {
    let t1 = Team;
    let t2 = Team;
}
