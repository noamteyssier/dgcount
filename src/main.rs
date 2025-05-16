use anyhow::{Result, bail};
use binseq::{BinseqReader, ParallelReader};
use clap::Parser;

mod cli;
mod count;
pub use cli::Args;
use count::CountDualGuides;

fn main() -> Result<()> {
    let args = Args::parse();
    let reader = BinseqReader::new(&args.binseq)?;
    if !reader.is_paired() {
        bail!("dgcount expects paired inputs.")
    }
    let proc = CountDualGuides::new();
    reader.process_parallel(proc, args.threads())?;
    Ok(())
}
