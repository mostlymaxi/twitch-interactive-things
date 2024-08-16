#[repr(C)]
pub struct LatestStreamInfo {
    pub msgs_per_15s: u64,
    pub msgs_per_30s: u64,
    pub msgs_per_60s: u64,
    pub waters_per_10m: u64,
    pub raid: u64,   // set to true for first 3 minutes of raid or something?
    pub follow: u64, // see above^ but for 15 seconds?
    pub redeem: u64,
}
