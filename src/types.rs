use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct HeapSummary {
    pub allocated_total: i64,
    pub frees: i64,
    pub allocations: i64,
    pub allocated_at_exit: i64,
    pub blocks_at_exit: i64,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct HeapSummaryHuman {
    pub allocated_total: String,
    pub frees: i64,
    pub allocations: i64,
    pub allocated_at_exit: String,
    pub blocks_at_exit: i64,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct LeakSummary {
    pub definitely_lost: i64,
    pub indirectly_lost: i64,
    pub possibly_lost: i64,
    pub still_reachable: i64,
    pub supressed: i64,
    pub definitely_lost_blocks: i64,
    pub indrectly_lost_blocks: i64,
    pub possibly_lost_blocks: i64,
    pub still_reachable_blocks: i64,
    pub supressed_blocks: i64,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct LeakSummaryHuman {
    pub definitely_lost: String,
    pub indirectly_lost: String,
    pub possibly_lost: String,
    pub still_reachable: String,
    pub supressed: String,
    pub definitely_lost_blocks: i64,
    pub indrectly_lost_blocks: i64,
    pub possibly_lost_blocks: i64,
    pub still_reachable_blocks: i64,
    pub supressed_blocks: i64,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct CacheMiss {
    pub l1i: f64,
    pub l1d: f64,
    pub lli: f64,
    pub lld: f64,
    pub llt: f64,
}
