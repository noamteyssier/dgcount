use std::collections::HashMap;
use std::sync::Arc;

use anyhow::{Result, bail};
use serde::Deserialize;

type ByteString = Vec<u8>;
type IndexPair = (usize, usize);

#[derive(Deserialize, Debug)]
struct GuideRecord {
    guide_pair_name: String,
    gene_pair_name: String,
    proto_a: String,
    proto_b: String,
}

#[derive(Clone)]
pub struct Library {
    /// Maps each unique protospacer to their protospacer index
    seqmap: HashMap<ByteString, usize>,
    /// Maps each unique index pair to their pair index
    pairmap: HashMap<IndexPair, usize>,
    /// Guide pair names
    guide_pairs: Vec<String>,
    /// Gene pair names
    gene_pairs: Vec<String>,
}
impl Library {
    pub fn new(path: &str) -> Result<Self> {
        let mut seqmap = HashMap::default();
        let mut pairmap = HashMap::default();
        let mut guide_pairs = Vec::default();
        let mut gene_pairs = Vec::default();

        let reader = csv::ReaderBuilder::new()
            .has_headers(false)
            .delimiter(b'\t')
            .from_path(path)?;

        for record in reader.into_deserialize() {
            let record: GuideRecord = record?;

            let tgt_i = if let Some(idx) = seqmap.get(record.proto_a.as_bytes()) {
                *idx
            } else {
                let idx = seqmap.len();
                seqmap.insert(record.proto_a.as_bytes().to_vec(), idx);
                idx
            };

            let tgt_j = if let Some(idx) = seqmap.get(record.proto_b.as_bytes()) {
                *idx
            } else {
                let idx = seqmap.len();
                seqmap.insert(record.proto_b.as_bytes().to_vec(), idx);
                idx
            };

            if pairmap.get(&(tgt_i, tgt_j)).is_some() {
                bail!("Duplicate protospacer pair found in record: {record:?}")
            } else {
                let pair_id = pairmap.len();
                pairmap.insert((tgt_i, tgt_j), pair_id);
            }

            guide_pairs.push(record.guide_pair_name);
            gene_pairs.push(record.gene_pair_name);
        }

        Ok(Self {
            seqmap,
            pairmap,
            guide_pairs,
            gene_pairs,
        })
    }

    pub fn new_arc(path: &str) -> Result<Arc<Self>> {
        Self::new(path).map(Arc::new)
    }
}
