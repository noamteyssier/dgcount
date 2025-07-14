# dgcount

Dual guide CRISPR counting

## Installation

```bash
cargo install dgcount
dgcount --help
```

## Usage

This expects as input a `library` file and at least one `sample` file (in [binseq](https://github.com/arcinstitute/binseq) format).

The `library` is a 4 column TSV without a header with columns:

1. guide pair name
2. gene pair name
3. protospacer 1
4. protospacer 2 (reverse complement)

This assumes that all protospacer sizes are equal.

The sample is expected to be paired and this assumes that the secondary sequence is reverse complemented from the sequencer.

The protospacer can be placed anywhere within the read in both the R1 and R2 and this is robust to stagger sequencing.

The algorithm first indexes the input library and finds expected protospacer pairs.
It then scans each record for protospacer sequences, if there is one in both primary and secondary sequence *and* that combination is expected, then the guide pair count is incremented.

Multiple samples can be provided at the CLI and a counts table is generated including all samples.

```bash
dgcount library.tsv sample1.bq sample2.bq sample3.bq
```

> Note: By default `dgcount` will match unambiguous one-off mismatches to the library. If you want only *perfect* matches then use the `-x`or `--exact` flag.
