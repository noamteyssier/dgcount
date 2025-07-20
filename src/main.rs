use std::time::Instant;

use anyhow::{Result, bail};
use binseq::ParallelReader;
use clap::Parser;
use paraseq::parallel::PairedParallelReader;

mod cli;
mod count;
mod library;

use cli::Args;
use count::{CountDualGuides, eprint_stats};
use library::Library;

fn main() -> Result<()> {
    // Load arguments
    let args = Args::parse();
    if args.binseq.is_empty() && args.fastq.is_empty() {
        bail!("Requires at least one input file. Run `--help` for CLI arguments")
    }

    // Initialize library
    let start = Instant::now();
    let library = if args.exact {
        Library::new_exact_arc(&args.library)?
    } else {
        Library::new_arc(&args.library)?
    };
    let elapsed = start.elapsed();
    eprintln!("Elapsed library initialization time: {:?}", elapsed);

    // Initialize output
    let mut output = args.output_handle()?;

    // Process readers
    let mut counts = Vec::default();
    let mut stats = Vec::default();

    let start = Instant::now();
    if args.fastq.is_empty() {
        // Initialize binseq readers
        let readers = args.readers()?;
        for reader in readers {
            let proc = CountDualGuides::new(library.clone());
            reader.process_parallel(proc.clone(), args.threads())?;
            counts.push(proc.counts());
            stats.push(proc.stats())
        }
    } else {
        // initialize fastq readers
        let reader_pairs = args.fastq_readers()?;
        for (rdr1, rdr2) in reader_pairs {
            let proc = CountDualGuides::new(library.clone());
            rdr1.process_parallel_paired(rdr2, proc.clone(), args.threads())?;
            counts.push(proc.counts());
            stats.push(proc.stats())
        }
    }
    let elapsed = start.elapsed();

    // Print output and stats
    library.pprint(&counts, &mut output)?;
    eprint_stats(&stats)?;
    println!("Elapsed map time: {:?}", elapsed);
    Ok(())
}
