//! Measurement protocol for determining renderable dimensions.
//!
//! This module provides the `Measurement` struct and associated functions
//! for calculating the minimum and maximum cell widths required to render
//! content in the terminal.

use std::cmp::{max, min};

/// Measurement of a renderable's width requirements.
///
/// A `Measurement` captures the minimum and maximum cell widths that a
/// renderable needs. The minimum is the tightest the content can be
/// compressed, while maximum is how wide it would be unconstrained.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Measurement {
    /// Minimum cells required (cannot render narrower).
    pub minimum: usize,
    /// Maximum cells required (ideal unconstrained width).
    pub maximum: usize,
}

impl Measurement {
    /// Create a new measurement.
    #[must_use]
    pub const fn new(minimum: usize, maximum: usize) -> Self {
        Self { minimum, maximum }
    }

    /// Create a measurement where min equals max.
    #[must_use]
    pub const fn exact(size: usize) -> Self {
        Self {
            minimum: size,
            maximum: size,
        }
    }

    /// Create a zero measurement.
    #[must_use]
    pub const fn zero() -> Self {
        Self {
            minimum: 0,
            maximum: 0,
        }
    }

    /// Get the span (difference) between minimum and maximum.
    #[must_use]
    pub const fn span(&self) -> usize {
        self.maximum.saturating_sub(self.minimum)
    }

    /// Normalize the measurement to ensure min <= max and both >= 0.
    #[must_use]
    pub fn normalize(&self) -> Self {
        let min_val = min(self.minimum, self.maximum);
        let max_val = max(self.minimum, self.maximum);
        Self {
            minimum: min_val,
            maximum: max_val,
        }
    }

    /// Constrain the maximum to a given width.
    ///
    /// Both minimum and maximum will be clamped to not exceed `width`.
    #[must_use]
    pub fn with_maximum(&self, width: usize) -> Self {
        Self {
            minimum: min(self.minimum, width),
            maximum: min(self.maximum, width),
        }
    }

    /// Ensure the minimum is at least `width`.
    ///
    /// Both minimum and maximum will be at least `width`.
    #[must_use]
    pub fn with_minimum(&self, width: usize) -> Self {
        Self {
            minimum: max(self.minimum, width),
            maximum: max(self.maximum, width),
        }
    }

    /// Clamp measurement to optional min/max bounds.
    #[must_use]
    pub fn clamp(&self, min_width: Option<usize>, max_width: Option<usize>) -> Self {
        let mut result = *self;
        if let Some(min_w) = min_width {
            result = result.with_minimum(min_w);
        }
        if let Some(max_w) = max_width {
            result = result.with_maximum(max_w);
        }
        result
    }

    /// Combine two measurements, taking the tighter constraints.
    ///
    /// The combined minimum is the max of both minimums,
    /// and the combined maximum is the max of both maximums.
    #[must_use]
    pub fn union(&self, other: &Self) -> Self {
        Self {
            minimum: max(self.minimum, other.minimum),
            maximum: max(self.maximum, other.maximum),
        }
    }

    /// Intersect two measurements, taking the overlapping range.
    ///
    /// Returns the intersection of the two ranges, or None if they don't overlap.
    #[must_use]
    pub fn intersect(&self, other: &Self) -> Option<Self> {
        let min_val = max(self.minimum, other.minimum);
        let max_val = min(self.maximum, other.maximum);

        if min_val <= max_val {
            Some(Self {
                minimum: min_val,
                maximum: max_val,
            })
        } else {
            None
        }
    }

    /// Add a constant width to both minimum and maximum.
    #[must_use]
    pub fn add(&self, width: usize) -> Self {
        Self {
            minimum: self.minimum.saturating_add(width),
            maximum: self.maximum.saturating_add(width),
        }
    }

    /// Subtract a constant width from both minimum and maximum.
    #[must_use]
    pub fn subtract(&self, width: usize) -> Self {
        Self {
            minimum: self.minimum.saturating_sub(width),
            maximum: self.maximum.saturating_sub(width),
        }
    }

    /// Check if a width fits within this measurement.
    #[must_use]
    pub fn fits(&self, width: usize) -> bool {
        width >= self.minimum && width <= self.maximum
    }
}

impl std::ops::Add for Measurement {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            minimum: self.minimum.saturating_add(rhs.minimum),
            maximum: self.maximum.saturating_add(rhs.maximum),
        }
    }
}

impl std::ops::AddAssign for Measurement {
    fn add_assign(&mut self, rhs: Self) {
        self.minimum = self.minimum.saturating_add(rhs.minimum);
        self.maximum = self.maximum.saturating_add(rhs.maximum);
    }
}

/// Combine multiple measurements by taking the union.
///
/// The resulting minimum is the max of all minimums (tightest constraint),
/// and the maximum is the max of all maximums (most flexible).
#[must_use]
pub fn measure_union(measurements: &[Measurement]) -> Measurement {
    if measurements.is_empty() {
        return Measurement::zero();
    }

    Measurement {
        minimum: measurements.iter().map(|m| m.minimum).max().unwrap_or(0),
        maximum: measurements.iter().map(|m| m.maximum).max().unwrap_or(0),
    }
}

/// Sum multiple measurements.
///
/// Useful for measuring horizontal concatenation of renderables.
#[must_use]
pub fn measure_sum(measurements: &[Measurement]) -> Measurement {
    if measurements.is_empty() {
        return Measurement::zero();
    }

    Measurement {
        minimum: measurements.iter().map(|m| m.minimum).sum(),
        maximum: measurements.iter().map(|m| m.maximum).sum(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_measurement_new() {
        let m = Measurement::new(5, 10);
        assert_eq!(m.minimum, 5);
        assert_eq!(m.maximum, 10);
    }

    #[test]
    fn test_measurement_exact() {
        let m = Measurement::exact(7);
        assert_eq!(m.minimum, 7);
        assert_eq!(m.maximum, 7);
        assert_eq!(m.span(), 0);
    }

    #[test]
    fn test_measurement_span() {
        let m = Measurement::new(5, 10);
        assert_eq!(m.span(), 5);
    }

    #[test]
    fn test_measurement_normalize() {
        let m = Measurement::new(10, 5); // Wrong order
        let normalized = m.normalize();
        assert_eq!(normalized.minimum, 5);
        assert_eq!(normalized.maximum, 10);
    }

    #[test]
    fn test_with_maximum() {
        let m = Measurement::new(5, 20);
        let constrained = m.with_maximum(10);
        assert_eq!(constrained.minimum, 5);
        assert_eq!(constrained.maximum, 10);
    }

    #[test]
    fn test_with_maximum_clamps_min() {
        let m = Measurement::new(15, 20);
        let constrained = m.with_maximum(10);
        assert_eq!(constrained.minimum, 10);
        assert_eq!(constrained.maximum, 10);
    }

    #[test]
    fn test_with_minimum() {
        let m = Measurement::new(5, 20);
        let constrained = m.with_minimum(10);
        assert_eq!(constrained.minimum, 10);
        assert_eq!(constrained.maximum, 20);
    }

    #[test]
    fn test_clamp() {
        let m = Measurement::new(3, 30);
        let clamped = m.clamp(Some(5), Some(20));
        assert_eq!(clamped.minimum, 5);
        assert_eq!(clamped.maximum, 20);
    }

    #[test]
    fn test_union() {
        let a = Measurement::new(5, 15);
        let b = Measurement::new(10, 12);
        let union = a.union(&b);
        assert_eq!(union.minimum, 10); // max of minimums
        assert_eq!(union.maximum, 15); // max of maximums
    }

    #[test]
    fn test_intersect() {
        let a = Measurement::new(5, 15);
        let b = Measurement::new(10, 20);
        let intersect = a.intersect(&b).unwrap();
        assert_eq!(intersect.minimum, 10);
        assert_eq!(intersect.maximum, 15);
    }

    #[test]
    fn test_intersect_no_overlap() {
        let a = Measurement::new(5, 10);
        let b = Measurement::new(15, 20);
        assert!(a.intersect(&b).is_none());
    }

    #[test]
    fn test_add_measurement() {
        let a = Measurement::new(5, 10);
        let b = Measurement::new(3, 7);
        let sum = a + b;
        assert_eq!(sum.minimum, 8);
        assert_eq!(sum.maximum, 17);
    }

    #[test]
    fn test_add_width() {
        let m = Measurement::new(5, 10);
        let added = m.add(3);
        assert_eq!(added.minimum, 8);
        assert_eq!(added.maximum, 13);
    }

    #[test]
    fn test_subtract_width() {
        let m = Measurement::new(5, 10);
        let subtracted = m.subtract(3);
        assert_eq!(subtracted.minimum, 2);
        assert_eq!(subtracted.maximum, 7);
    }

    #[test]
    fn test_fits() {
        let m = Measurement::new(5, 10);
        assert!(!m.fits(4));
        assert!(m.fits(5));
        assert!(m.fits(7));
        assert!(m.fits(10));
        assert!(!m.fits(11));
    }

    #[test]
    fn test_measure_union() {
        let measurements = vec![
            Measurement::new(5, 10),
            Measurement::new(3, 15),
            Measurement::new(8, 12),
        ];
        let union = measure_union(&measurements);
        assert_eq!(union.minimum, 8);  // max of minimums
        assert_eq!(union.maximum, 15); // max of maximums
    }

    #[test]
    fn test_measure_union_empty() {
        let union = measure_union(&[]);
        assert_eq!(union.minimum, 0);
        assert_eq!(union.maximum, 0);
    }

    #[test]
    fn test_measure_sum() {
        let measurements = vec![
            Measurement::new(5, 10),
            Measurement::new(3, 7),
        ];
        let sum = measure_sum(&measurements);
        assert_eq!(sum.minimum, 8);
        assert_eq!(sum.maximum, 17);
    }
}
