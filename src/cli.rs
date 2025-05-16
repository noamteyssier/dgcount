use std::{
    fs::File,
    io::{BufWriter, Write},
};

use anyhow::{Result, bail};
use binseq::BinseqReader;
use clap::Parser;

#[derive(Parser, Debug)]
pub struct Args {
    /// Input library (3-column tsv) no header
    ///
    /// [gene, guide1, guide2]
    pub library: String,

    /// Input files (*.bq / *.vbq)
    pub binseq: Vec<String>,

    #[clap(short, long, default_value = "dgcount.out.tsv")]
    pub output: String,

    #[clap(short = 'T', long, default_value_t = 0)]
    threads: usize,
}
impl Args {
    pub fn threads(&self) -> usize {
        if self.threads == 0 {
            num_cpus::get()
        } else {
            self.threads.min(num_cpus::get())
        }
    }

    pub fn readers(&self) -> Result<Vec<BinseqReader>> {
        let mut readers = Vec::default();
        for path in self.binseq.iter() {
            let reader = BinseqReader::new(path)?;
            if !reader.is_paired() {
                bail!("dgcount expects paired inputs.")
            }
            readers.push(reader);
        }
        Ok(readers)
    }

    pub fn output_handle(&self) -> Result<Box<dyn Write>> {
        let handle = File::create(&self.output).map(BufWriter::new)?;
        Ok(Box::new(handle))
    }
}
