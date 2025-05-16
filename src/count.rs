use binseq::{BinseqRecord, ParallelProcessor};

#[derive(Clone)]
pub struct CountDualGuides {
    sbuf: Vec<u8>,
    xbuf: Vec<u8>,
}
impl CountDualGuides {
    pub fn new() -> Self {
        Self {
            sbuf: Vec::default(),
            xbuf: Vec::default(),
        }
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
}
impl ParallelProcessor for CountDualGuides {
    fn process_record<R: BinseqRecord>(&mut self, record: R) -> binseq::Result<()> {
        self.decode_record(&record)?;
        Ok(())
    }

    fn on_batch_complete(&mut self) -> binseq::Result<()> {
        Ok(())
    }
}
