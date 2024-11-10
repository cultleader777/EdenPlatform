use std::error::Error;
use crate::common::*;

#[tokio::test]
async fn moonbeam_metrics_gathered() -> Result<(), Box<dyn Error>> {
    assert!(does_prometheus_metric_exist("10.17.0.10", 9090, "moonbeam_frontier_eth_blocks_cache_hits").await?);

    Ok(())
}
