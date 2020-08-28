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

pub struct Measurements {
    duration: Duration,
    size: Address,
    samples: usize,
    measures: BTreeSet<StateFreq>,
    min_percentile: f32,
    max_display: Option<usize>,
}

impl Measurements {
    pub(crate) fn new(duration: Duration, size: Address, samples: usize, measures: HashMap<u64, usize>) -> Measurements {
        let measures = {
            let mut res = BTreeSet::new();
            for (state, freq) in measures {
                res.insert(StateFreq::new(freq, state, samples));
            }
            res
        };

        let min_percentile = 1.0;
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

    pub fn duration(&self) -> Duration {
        self.duration
    }

    pub fn format_options(&mut self, min_percentile: f32, max_display: Option<usize>) {
        self.max_display = max_display;
        self.min_percentile = min_percentile;
    }
}

impl fmt::Display for Measurements {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f, 
            "[Measurements obtained in {} ms]\n[Sample count of {}]\n[Results:\n", 
            self.duration.as_millis(),
            self.samples,
        ).unwrap();

        let count = match self.max_display {
            Some(max) => max,
            None => self.samples,
        };

        for (i, pair) in self.measures.iter().enumerate() {
            if pair.percent < self.min_percentile || i == count {
                write!(f, "   and {} more...\n", self.measures.len() - i).unwrap();
                break;
            }

            write!(
                f,
                "  |{:0size$b}> => {:5.2}%,\n",
                pair.state,
                pair.percent,
                size = self.size as usize,
            ).unwrap();
        }

        if count != self.samples {

        }

        write!(f, "]")
    }
}