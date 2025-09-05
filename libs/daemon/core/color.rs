use dashmap::DashSet;
use rand::{distr::weighted::WeightedIndex, prelude::*};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

/// Calculates the shortest distance between two numbers on a circle.
/// The circle's circumference is `max_val + 1`.
fn circular_distance(a: u32, b: u32, max_val: u32) -> u32 {
    let circumference = max_val + 1;
    let diff = a.abs_diff(b);
    let wrap_around_diff = circumference - diff;
    diff.min(wrap_around_diff)
}

/// A thread-safe random number generator that attempts to pick numbers that are far away from
/// previously picked numbers. It uses a layering system to ensure all numbers
/// are picked before any number is picked a second time, and so on.
pub struct SpacedRandomGenerator {
    max_val: u32,
    intensity: f64,
    /// Atomic pick counts for each number
    pick_counts: Arc<[AtomicUsize]>,
    /// Thread-safe set of numbers that have been picked at least once
    picked_set: Arc<DashSet<u32>>,
}

impl SpacedRandomGenerator {
    /// Creates a new thread-safe generator.
    ///
    /// # Arguments
    ///
    /// * `max_val` - The maximum number (inclusive) that can be generated (0-x).
    /// * `intensity` - A float that controls the bias.
    ///   - `0.0`: No bias, purely random selection from the least-picked numbers.
    ///   - `1.0`: Gentle bias towards farther numbers.
    ///   - `10.0+`: Strong bias, will almost always pick the mathematically farthest number.
    pub fn new(max_val: u32, intensity: f64) -> Self {
        let pick_counts: Vec<AtomicUsize> = (0..=max_val).map(|_| AtomicUsize::new(0)).collect();

        SpacedRandomGenerator {
            max_val,
            intensity,
            pick_counts: Arc::from(pick_counts),
            picked_set: Arc::new(DashSet::new()),
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
    pub fn push(&self, number: u32) -> bool {
        if number > self.max_val {
            return false;
        }

        let idx = number as usize;
        let old_count = self.pick_counts[idx].fetch_add(1, Ordering::SeqCst);

        // Only add to picked_set if this is the first time
        if old_count == 0 {
            self.picked_set.insert(number);
        }

        true
    }

    /// Generates the next number.
    ///
    /// The generator first finds the numbers that have been picked the fewest times
    /// (the "lowest layer"). From that set of candidates, it picks one that is
    /// biased to be far away from all previously generated or pushed numbers.
    pub fn next_number(&self) -> u32 {
        // Step 1: Find the minimum pick count (the current lowest "layer").
        let min_picks = self.find_min_picks();

        // Step 2: Identify all numbers that are in this lowest layer.
        let candidates: Vec<u32> = (0..=self.max_val)
            .filter(|&n| self.pick_counts[n as usize].load(Ordering::SeqCst) == min_picks)
            .collect();

        // Rule: If no numbers have been generated or pushed yet, pick a truly
        // random one from the candidates (which is all numbers at this point).
        if self.picked_set.is_empty() {
            let choice = candidates[rand::rng().random_range(0..candidates.len())];
            self.update_pick_count(choice);
            return choice;
        }

        // Step 3: Calculate a "farness score" for each candidate against the picked set.
        let weights: Vec<f64> = candidates
            .iter()
            .map(|&candidate| {
                // The score is the minimum distance to any number in the picked set.
                let min_dist = self
                    .picked_set
                    .iter()
                    .map(|x| circular_distance(candidate, *x, self.max_val))
                    .min()
                    .unwrap_or(0); // Should not happen with non-empty set, but safe.
                                   // Convert score to a probability weight using the intensity.
                (min_dist as f64 + 1e-9).powf(self.intensity)
            })
            .collect();

        // Step 4: Use a weighted distribution to pick the next number.
        let dist = WeightedIndex::new(&weights).expect("Could not create weighted distribution");
        let chosen_index = dist.sample(&mut rand::rng());
        let chosen_number = candidates[chosen_index];

        // Step 5: Update our state and return the chosen number.
        self.update_pick_count(chosen_number);
        chosen_number
    }

    /// Helper function to find the minimum pick count
    fn find_min_picks(&self) -> usize {
        self.pick_counts
            .iter()
            .map(|count| count.load(Ordering::SeqCst))
            .min()
            .unwrap_or(0)
    }

    /// Helper function to update pick count and picked set
    fn update_pick_count(&self, number: u32) {
        let idx = number as usize;
        let old_count = self.pick_counts[idx].fetch_add(1, Ordering::SeqCst);

        // Only add to picked_set if this is the first time
        if old_count == 0 {
            self.picked_set.insert(number);
        }
    }
}
