use std::io::stderr;
use std::ops::AddAssign;
use std::sync::Arc;

use anyhow::Result;
use binseq::{BinseqRecord, ParallelProcessor};
use parking_lot::Mutex;
use serde::Serialize;

use crate::library::{Counts, Library};

#[derive(Clone, Copy, Debug, Default, Serialize)]
pub struct Statistics {
    n_records: usize,
    n_mapped: usize,
    missing_a: usize,
    missing_b: usize,
}
impl Statistics {
    fn reset(&mut self) {
        self.n_records = 0;
        self.n_mapped = 0;
        self.missing_a = 0;
        self.missing_b = 0;
    }
}
impl AddAssign for Statistics {
    fn add_assign(&mut self, rhs: Self) {
        self.n_records += rhs.n_records;
        self.n_mapped += rhs.n_mapped;
        self.missing_a += rhs.missing_a;
        self.missing_b += rhs.missing_b;
    }
}

pub fn eprint_stats(stats: &[Statistics]) -> Result<()> {
    let mut csv_writer = csv::WriterBuilder::default()
        .delimiter(b'\t')
        .has_headers(true)
        .from_writer(stderr());
    for stat in stats {
        csv_writer.serialize(stat)?;
    }
    csv_writer.flush()?;

    Ok(())
}

#[derive(Clone)]
pub struct CountDualGuides {
    sbuf: Vec<u8>,
    xbuf: Vec<u8>,
    library: Arc<Library>,
    local_counts: Counts,
    local_stats: Statistics,
    global_counts: Arc<Mutex<Counts>>,
    global_stats: Arc<Mutex<Statistics>>,
}
impl CountDualGuides {
    pub fn new(library: Arc<Library>) -> Self {
        Self {
            sbuf: Vec::default(),
            xbuf: Vec::default(),
            local_counts: library.build_counts(),
            global_counts: Arc::new(Mutex::new(library.build_counts())),
            local_stats: Statistics::default(),
            global_stats: Arc::new(Mutex::new(Statistics::default())),
            library,
        }
    }

    pub fn counts(&self) -> Counts {
        self.global_counts.lock().clone()
    }

    pub fn stats(&self) -> Statistics {
        *self.global_stats.lock()
    }

    fn clear_buffers(&mut self) {
        self.sbuf.clear();
        self.xbuf.clear();
    }

    fn decode_record<R: BinseqRecord>(&mut self, ref record: R) -> binseq::Result<()> {
        self.clear_buffers();
        record.decode_s(&mut self.sbuf)?;
        record.decode_x(&mut self.xbuf)?;
        Ok(())
    }

    fn match_protospacer(&self, buffer: &[u8]) -> Option<usize> {
        for subseq in buffer.windows(self.library.slen) {
            if let Some(tgt) = self.library.contains_protospacer(subseq) {
                return Some(tgt);
            }
        }
        None
    }

    fn match_pair(&self, i: usize, j: usize) -> Option<usize> {
        self.library.contains_pair(i, j)
    }
}
impl ParallelProcessor for CountDualGuides {
    fn process_record<R: BinseqRecord>(&mut self, record: R) -> binseq::Result<()> {
        self.decode_record(&record)?;
        self.local_stats.n_records += 1;

        match (
            self.match_protospacer(&self.sbuf),
            self.match_protospacer(&self.xbuf),
        ) {
            (Some(i), Some(j)) => {
                self.match_pair(i, j).map(|p_idx| {
                    self.local_stats.n_mapped += 1;
                    self.local_counts.inc(p_idx)
                });
            }
            (Some(_), None) => {
                self.local_stats.missing_b += 1;
            }
            (None, Some(_)) => {
                self.local_stats.missing_a += 1;
            }
            (None, None) => {}
        }
        Ok(())
    }

    fn on_batch_complete(&mut self) -> binseq::Result<()> {
        {
            self.global_counts.lock().ingest(&self.local_counts);
        } // drop lock

        {
            *self.global_stats.lock() += self.local_stats;
        } // drop lock

        self.local_counts.reset();
        self.local_stats.reset();
        Ok(())
    }
}
