use rand::{distr::weighted::WeightedIndex, prelude::*};

/// Calculates the shortest distance between two numbers on a circle.
/// The circle's circumference is `max_val + 1`.
fn circular_distance(a: u32, b: u32, max_val: u32) -> u32 {
    let circumference = max_val + 1;
    let diff = a.abs_diff(b);
    let wrap_around_diff = circumference - diff;
    diff.min(wrap_around_diff)
}

/// A random number generator that attempts to pick numbers that are far away from
/// previously picked numbers. It uses a layering system to ensure all numbers
/// are picked before any number is picked a second time, and so on.
pub struct SpacedRandomGenerator {
    max_val: u32,
    intensity: f64,
    /// Stores the number of times each number (by index) has been picked.
    /// This forms the basis of the "layering" system.
    pick_counts: Vec<usize>,
    /// Stores the historical sequence of picked/pushed numbers for distance calculations.
    generation_sequence: Vec<u32>,
    rng: ThreadRng,
}

impl SpacedRandomGenerator {
    /// Creates a new generator.
    ///
    /// # Arguments
    ///
    /// * `max_val` - The maximum number (inclusive) that can be generated (0-x).
    /// * `intensity` - A float that controls the bias.
    ///   - `0.0`: No bias, purely random selection from the least-picked numbers.
    ///   - `1.0`: Gentle bias towards farther numbers.
    ///   - `10.0+`: Strong bias, will almost always pick the mathematically farthest number.
    pub fn new(max_val: u32, intensity: f64) -> Self {
        SpacedRandomGenerator {
            max_val,
            intensity,
            // All numbers start with a pick count of 0.
            pick_counts: vec![0; max_val as usize + 1],
            generation_sequence: Vec::new(),
            rng: rand::rng(),
        }
    }

    /// Manually adds a number to the sequence, incrementing its pick count.
    ///
    /// This influences future selections by making the pushed number part of a
    /// "higher layer" and using it as a reference point for spacing.
    ///
    /// # Returns
    ///
    /// * `true` if the number was successfully pushed.
    /// * `false` if the number was out of bounds (`> max_val`).
    pub fn push(&mut self, number: u32) -> bool {
        if number > self.max_val {
            return false;
        }
        let number_idx = number as usize;
        self.pick_counts[number_idx] += 1;
        self.generation_sequence.push(number);
        true
    }

    /// Generates the next number.
    ///
    /// The generator first finds the numbers that have been picked the fewest times
    /// (the "lowest layer"). From that set of candidates, it picks one that is
    /// biased to be far away from all previously generated or pushed numbers.
    pub fn next_number(&mut self) -> u32 {
        // Step 1: Find the minimum pick count (the current lowest "layer").
        // This unwrap is safe because pick_counts is never empty.
        let min_picks = *self.pick_counts.iter().min().unwrap();

        // Step 2: Identify all numbers that are in this lowest layer.
        let candidates: Vec<u32> = (0..=self.max_val)
            .filter(|&n| self.pick_counts[n as usize] == min_picks)
            .collect();

        // Rule: If no numbers have been generated or pushed yet, pick a truly
        // random one from the candidates (which is all numbers at this point).
        if self.generation_sequence.is_empty() {
            let choice = candidates[self.rng.gen_range(0..candidates.len())];
            self.pick_counts[choice as usize] += 1;
            self.generation_sequence.push(choice);
            return choice;
        }

        // Step 3: Calculate a "farness score" for each candidate against the entire history.
        let weights: Vec<f64> = candidates
            .iter()
            .map(|&candidate| {
                // The score is the minimum distance to any number picked in the entire history.
                let min_dist = self
                    .generation_sequence
                    .iter()
                    .map(|&picked| circular_distance(candidate, picked, self.max_val))
                    .min()
                    .unwrap_or(0); // Should not happen with non-empty sequence, but safe.

                // Convert score to a probability weight using the intensity.
                (min_dist as f64 + 1e-9).powf(self.intensity)
            })
            .collect();

        // Step 4: Use a weighted distribution to pick the next number.
        let dist = WeightedIndex::new(&weights).expect("Could not create weighted distribution");
        let chosen_index = dist.sample(&mut self.rng);
        let chosen_number = candidates[chosen_index];

        // Step 5: Update our state and return the chosen number.
        self.pick_counts[chosen_number as usize] += 1;
        self.generation_sequence.push(chosen_number);
        chosen_number
    }

    /// Returns a slice of the full historical sequence of numbers that have been
    /// generated or pushed.
    pub fn get_picked_sequence(&self) -> &[u32] {
        &self.generation_sequence
    }

    /// Returns the pick count for a specific number.
    pub fn get_pick_count(&self, number: u32) -> Option<usize> {
        self.pick_counts.get(number as usize).copied()
    }

    /// Resets the generator to its initial state, clearing all pick counts and history.
    pub fn reset(&mut self) {
        self.pick_counts.fill(0);
        self.generation_sequence.clear();
    }
}

/// For a given pick number `n`, calculates the distance a "perfect"
/// spacing algorithm would achieve for that pick.
pub fn get_ideal_distance(num_picked: usize, max_val: u32) -> f64 {
    if num_picked <= 1 {
        return 0.0; // No distance to compare against for the first pick
    }
    let circumference = max_val as f64;
    let max_possible_dist = circumference / 2.0;

    // The divisor follows the sequence 1, 2, 2, 4, 4, 4, 4, 8, ...
    // This can be calculated as 2 to the power of the floor of log2(n-1).
    let n = num_picked;
    let divisor = 2.0_f64.powi((n - 1).ilog2() as i32);

    max_possible_dist / divisor
}
