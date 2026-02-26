use std::collections::HashMap;
use std::io::Write;
use std::sync::Arc;

use anyhow::{Result, bail};
use seqhash::{SeqHash, SeqHashBuilder};
use serde::Deserialize;

type IndexPair = (usize, usize);

#[derive(Deserialize, Debug)]
struct GuideRecord {
    guide_pair_name: String,
    gene_pair_name: String,
    proto_a: String,
    proto_b: String,
}

#[derive(Clone, Debug)]
pub struct Counts {
    inner: Vec<usize>,
}
impl Counts {
    pub fn new(size: usize) -> Self {
        Self {
            inner: vec![0; size],
        }
    }

    /// Increment a specific index by one
    ///
    /// Will panic if out of range
    pub fn inc(&mut self, idx: usize) {
        self.inner[idx] += 1;
    }

    /// Take all counts from the rhs
    pub fn ingest(&mut self, rhs: &Self) {
        assert_eq!(
            self.inner.len(),
            rhs.inner.len(),
            "Mismatch in counts vector size :: error in init"
        );

        self.inner
            .iter_mut()
            .zip(rhs.inner.iter())
            .for_each(|(i, j)| *i += j);
    }

    /// Reset all internal counts to zero
    pub fn reset(&mut self) {
        self.inner.iter_mut().for_each(|x| *x = 0);
    }
}

#[derive(Clone)]
pub struct Library {
    /// Maps each unique index pair to their pair index
    pairmap: HashMap<IndexPair, usize>,
    /// Guide pair names
    guide_pairs: Vec<String>,
    /// Gene pair names
    gene_pairs: Vec<String>,
    /// Disambiguation sequence
    seqhash: SeqHash,
}
impl Library {
    pub fn new(path: &str, exact: bool) -> Result<Self> {
        let mut seqmap = HashMap::default();
        let mut parents = Vec::default();
        let mut pairmap = HashMap::default();
        let mut guide_pairs = Vec::default();
        let mut gene_pairs = Vec::default();
        let mut slen = None;

        let reader = csv::ReaderBuilder::new()
            .has_headers(false)
            .delimiter(b'\t')
            .from_path(path)?;

        // Inserts a sequence into the global sequence hashmap and returns its unique sequence index
        let insert_to_seqmap =
            |seq: &[u8], seqmap: &mut HashMap<Vec<u8>, usize>, parents: &mut Vec<_>| {
                if let Some(idx) = seqmap.get(seq) {
                    *idx
                } else {
                    let idx = seqmap.len();
                    seqmap.insert(seq.to_vec(), idx);
                    parents.push(seq.to_vec());
                    idx
                }
            };

        for record in reader.into_deserialize() {
            let record: GuideRecord = record?;

            if slen.is_none() {
                slen = Some(record.proto_a.len());
            } else {
                if record.proto_a.len() != slen.unwrap() || record.proto_b.len() != slen.unwrap() {
                    bail!("Size mismatch found in record: {record:?}");
                }
            }

            let tgt_i = insert_to_seqmap(record.proto_a.as_bytes(), &mut seqmap, &mut parents);
            let tgt_j = insert_to_seqmap(record.proto_b.as_bytes(), &mut seqmap, &mut parents);

            if pairmap.contains_key(&(tgt_i, tgt_j)) {
                bail!("Duplicate protospacer pair found in record: {record:?}")
            } else {
                let pair_id = pairmap.len();
                pairmap.insert((tgt_i, tgt_j), pair_id);
            }

            guide_pairs.push(record.guide_pair_name);
            gene_pairs.push(record.gene_pair_name);
        }

        // Get all unique protospacer sequences

        // Create all unambiguous one-off mismatches
        let mut builder = SeqHashBuilder::default();
        if exact {
            builder = builder.exact();
        }
        let seqhash = builder.build(&parents)?;

        Ok(Self {
            pairmap,
            guide_pairs,
            gene_pairs,
            seqhash,
        })
    }

    pub fn new_arc(path: &str, exact: bool) -> Result<Arc<Self>> {
        Self::new(path, exact).map(Arc::new)
    }

    pub fn build_counts(&self) -> Counts {
        Counts::new(self.pairmap.len())
    }

    /// Returns the index of the protospacer sequence after disambiguation
    pub fn contains_protospacer(&self, seq: &[u8]) -> Option<usize> {
        self.seqhash
            .query_sliding(seq)
            .map(|(mat, _pos)| mat.parent_idx())
    }

    pub fn contains_pair(&self, i: usize, j: usize) -> Option<usize> {
        self.pairmap.get(&(i, j)).copied()
    }

    pub fn pprint<W: Write>(&self, counts: &[Counts], output: &mut W) -> Result<()> {
        for v in counts {
            assert_eq!(
                v.inner.len(),
                self.guide_pairs.len(),
                "Size mismatch between counts and guide_pairs"
            );
            assert_eq!(
                v.inner.len(),
                self.gene_pairs.len(),
                "Size mismatch between counts and gene_pairs"
            );
        }

        for (idx, (guide_pair, gene_pair)) in self
            .guide_pairs
            .iter()
            .zip(self.gene_pairs.iter())
            .enumerate()
        {
            write!(output, "{guide_pair}\t{gene_pair}")?;
            for v in counts {
                output.write_all(format!("\t{}", v.inner[idx]).as_bytes())?;
            }
            output.write_all(b"\n")?;
        }

        output.flush()?;
        Ok(())
    }
}
