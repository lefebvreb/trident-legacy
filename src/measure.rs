use std::collections::{BTreeSet, HashMap};
use std::cmp::{Eq, Ord, Ordering};
use std::fmt;
use std::time::Duration;

use crate::computer::Address;

#[derive(PartialEq, PartialOrd)]
struct StateFreq {
    freq: usize,
    state: u64,
    percent: f32,
}

impl StateFreq {
    fn new(freq: usize, state: u64, samples: usize) -> StateFreq {
        StateFreq {
            freq,
            state,
            percent: freq as f32 * 100.0 / samples as f32,
        }
    }
}

impl Eq for StateFreq {}

impl Ord for StateFreq {
    #[inline]
    fn cmp(&self, other: &StateFreq) -> Ordering {
        if self.freq == other.freq {
            self.state.cmp(&other.state)
        } else {
            other.freq.cmp(&self.freq)
        }        
    }
}

/// Holds every information and results of a previous computation.
pub struct Measurements {
    duration: Duration,
    size: Address,
    samples: usize,
    measures: BTreeSet<StateFreq>,
    min_percentile: Option<f32>,
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
            for (state, freq) in measures {
                res.insert(StateFreq::new(freq, state, samples));
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

    /// Returns the total duration of the computation
    pub fn duration(&self) -> Duration {
        self.duration
    }

    /// Returns the `n` most frequent states measured, from most frequent to least frequent.
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
    /// Leave them to `None` to disable them.
    pub fn format_options<F, I>(&mut self, min_percentile: F, max_display: I) 
    where
        F: Into<Option<f32>>,
        I: Into<Option<usize>>,
    {
        self.min_percentile = min_percentile.into();
        self.max_display = max_display.into();        
    }
}

impl fmt::Display for Measurements {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f, 
            "[Measurements obtained in {} ms]\n[Sample count of {}]\n[Top results:\n", 
            self.duration.as_millis(),
            self.samples,
        ).unwrap();

        let min = match self.min_percentile {
            Some(min) => min,
            None => 0.0,
        };
        let max = match self.max_display {
            Some(max) => max,
            None => self.samples,
        };

        for (i, pair) in self.measures.iter().enumerate() {
            if pair.percent < min || i == max {
                write!(f, "    and {} more...\n", self.measures.len() - i).unwrap();
                break;
            }

            write!(
                f,
                "    |{:0size$b}> ~> {:5.2}%,\n",
                pair.state,
                pair.percent,
                size = self.size as usize,
            ).unwrap();
        }

        write!(f, "]")
    }
}