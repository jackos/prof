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
    pub i1_miss: f64,
    pub l2i_miss: f64,
    pub d1_miss: f64,
    pub l2d_miss: f64,
    pub l2_miss: f64,
}
