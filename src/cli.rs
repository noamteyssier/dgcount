use clap::Parser;

#[derive(Parser, Debug)]
pub struct Args {
    /// Input library (3-column tsv) no header
    ///
    /// [gene, guide1, guide2]
    pub library: String,

    /// Input file (*.bq / *.vbq)
    pub binseq: String,

    #[clap(short, long, default_value = "out.tsv")]
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
}
