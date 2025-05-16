use anyhow::{Result, bail};
use binseq::ParallelReader;
use clap::Parser;

mod cli;
mod count;
mod library;

use cli::Args;
use count::CountDualGuides;
use library::Library;

fn main() -> Result<()> {
    // Load arguments
    let args = Args::parse();
    if args.binseq.len() < 1 {
        bail!("Requires at least one input file. Run `--help` for CLI arguments")
    }

    // Initialize library
    let library = Library::new_arc(&args.library)?;

    // Initialize readers
    let readers = args.readers()?;

    // Initialize output
    let mut output = args.output_handle()?;

    // Process readers
    let mut counts = Vec::default();
    let mut stats = Vec::default();
    for reader in readers {
        let proc = CountDualGuides::new(library.clone());
        reader.process_parallel(proc.clone(), args.threads())?;
        counts.push(proc.counts());
        stats.push(proc.stats())
    }

    // Print output and stats
    library.pprint(&counts, &mut output)?;
    stats.iter().for_each(|x| eprintln!("{x:#?}"));
    Ok(())
}
