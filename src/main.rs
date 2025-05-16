use anyhow::{Result, bail};
use binseq::{BinseqReader, ParallelReader};
use clap::Parser;

mod cli;
mod count;
mod library;

use cli::Args;
use count::CountDualGuides;
use library::Library;

fn main() -> Result<()> {
    let args = Args::parse();
    let library = Library::new_arc(&args.library)?;
    let reader = BinseqReader::new(&args.binseq)?;
    if !reader.is_paired() {
        bail!("dgcount expects paired inputs.")
    }
    let proc = CountDualGuides::new(library);
    reader.process_parallel(proc.clone(), args.threads())?;
    Ok(())
}
