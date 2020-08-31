use std::collections::{BTreeSet, HashMap};
use std::cmp::{Eq, Ord, Ordering};
use std::fmt;
use std::time::Duration;

use crate::computer::Address;

//#################################################################################################
//
//                                      Measurement type
//
//#################################################################################################

#[derive(PartialEq, PartialOrd)]
struct Measurement {
    count: usize,
    state: u64,
    frequency: f64,
}

impl Eq for Measurement {}

impl Ord for Measurement {
    #[inline]
    fn cmp(&self, other: &Measurement) -> Ordering {
        match other.count.cmp(&self.count) {
            Ordering::Equal => self.state.cmp(&other.state),
            unequal => unequal,
        }   
    }
}

/// Holds every information and results of a previous computation.
pub struct Measurements {
    duration: Duration,
    size: Address,
    samples: usize,
    measures: BTreeSet<Measurement>,
    min_percentile: Option<f64>,
    max_display: Option<usize>,
}

impl Measurements {
    pub(crate) fn new(
        duration: Duration, 
        size: Address, 
        samples: usize, 
        measures: HashMap<u64, usize>
    ) -> Measurements {
        let measures = {
            let mut res = BTreeSet::new();
            for (state, count) in measures {
                res.insert(Measurement {
                    count, 
                    state, 
                    frequency: count as f64 / samples as f64,
                });
            }
            res
        };

        let min_percentile = None;
        let max_display = Some(25);

        Measurements {
            duration,
            size,
            samples,
            measures,
            max_display, 
            min_percentile,
        }
    }

    /// Returns the total duration of the computation.
    pub fn duration(&self) -> Duration {
        self.duration
    }

    /// Returns the `n` most frequent states measured, and their frequency of apparition,
    /// from most frequent to least frequent.
    /// If they was less than `n` different states measured, returns all of them.
    pub fn n_most(&self, n: usize) -> Box<[u64]> {
        self.measures.iter()
            .map(|pair| pair.state)
            .take(n)
            .collect()
    }

    /// Specifies the options for formatting the results:
    /// - `min_percentile` is the minimal percentile that results need to have been measured with
    /// in order to be displayed (default: `None`).
    /// - `max_display` is the maximum number of states that will be displayed. The rest will be
    /// hidden (default: `25`).
    /// 
    /// Leave either or both to `None` to disable them.
    pub fn format_options<F, I>(&mut self, min_percentile: F, max_display: I) 
    where
        F: Into<Option<f64>>,
        I: Into<Option<usize>>,
    {
        self.min_percentile = min_percentile.into();
        self.max_display = max_display.into();        
    }
}

impl fmt::Display for Measurements {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, 
            "[\n  [Measurements obtained in {} ms],\n  [Sample count of {}],\n  [Top results:\n", 
            self.duration.as_millis(),
            self.samples,
        ).unwrap();

        let len = self.measures.len();

        let min = match self.min_percentile {
            Some(min) => min,
            None => 0.0,
        };

        let max = match self.max_display {
            Some(max) => max,
            None => self.samples,
        };

        for (i, pair) in self.measures.iter().enumerate() {
            if pair.frequency < min || i == max {
                write!(f, "    and {} more...\n", len - i).unwrap();
                break;
            }

            write!(f,
                "    |{:0size$b}> ~> {:5.2}%{}\n",
                pair.state,
                pair.frequency * 100.0,
                if i+1 == len {""} else {","},
                size = self.size as usize,
            ).unwrap();
        }

        write!(f, "  ]\n]")
    }
}