use std::sync::Arc;

use binseq::{BinseqRecord, ParallelProcessor};
use parking_lot::Mutex;

use crate::library::{Counts, Library};

#[derive(Clone)]
pub struct CountDualGuides {
    sbuf: Vec<u8>,
    xbuf: Vec<u8>,
    library: Arc<Library>,
    local_counts: Counts,
    global_counts: Arc<Mutex<Counts>>,
}
impl CountDualGuides {
    pub fn new(library: Arc<Library>) -> Self {
        Self {
            sbuf: Vec::default(),
            xbuf: Vec::default(),
            local_counts: library.build_counts(),
            global_counts: Arc::new(Mutex::new(library.build_counts())),
            library,
        }
    }

    pub fn counts(&self) -> Counts {
        self.global_counts.lock().clone()
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
        for subseq in buffer.chunks(self.library.slen) {
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

        if let Some(tgt_i) = self.match_protospacer(&self.sbuf)
            && let Some(tgt_j) = self.match_protospacer(&self.xbuf)
        {
            self.match_pair(tgt_i, tgt_j)
                .map(|p_idx| self.local_counts.inc(p_idx));
        }

        Ok(())
    }

    fn on_batch_complete(&mut self) -> binseq::Result<()> {
        {
            self.global_counts.lock().ingest(&self.local_counts);
        } // drop lock
        self.local_counts.clear();
        Ok(())
    }
}
