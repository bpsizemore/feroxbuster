use super::*;
use crate::FeroxSerialize;
use anyhow::Result;
use reqwest::StatusCode;
use std::sync::Arc;
use tempfile::NamedTempFile;
use tokio::{sync::mpsc::UnboundedSender, task::JoinHandle};

/// simple helper to reduce code reuse
pub fn setup_stats_test() -> (
    Arc<Stats>,
    UnboundedSender<StatCommand>,
    JoinHandle<Result<()>>,
) {
    initialize()
}

/// another helper to stay DRY; must be called after any sent commands and before any checks
/// performed against the Stats object
pub async fn teardown_stats_test(
    sender: UnboundedSender<StatCommand>,
    handle: JoinHandle<Result<()>>,
) {
    // send exit and await, once the await completes, stats should be updated
    sender.send(StatCommand::Exit).unwrap_or_default();
    handle.await.unwrap().unwrap();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
/// when sent StatCommand::Exit, function should exit its while loop (runs forever otherwise)
async fn statistics_handler_exits() {
    let (_, sender, handle) = setup_stats_test();

    sender.send(StatCommand::Exit).unwrap_or_default();

    handle.await.unwrap().unwrap(); // blocks on the handler's while loop

    // if we've made it here, the test has succeeded
}

#[test]
/// Stats::save should write contents of Stats to disk
fn save_writes_stats_object_to_disk() {
    let stats = Stats::new();
    stats.add_request();
    stats.add_request();
    stats.add_request();
    stats.add_request();
    stats.add_error(StatError::Timeout);
    stats.add_error(StatError::Timeout);
    stats.add_error(StatError::Timeout);
    stats.add_error(StatError::Timeout);
    stats.add_status_code(StatusCode::OK);
    stats.add_status_code(StatusCode::OK);
    stats.add_status_code(StatusCode::OK);
    let outfile = NamedTempFile::new().unwrap();
    if stats
        .save(174.33, &outfile.path().to_str().unwrap())
        .is_ok()
    {}

    assert!(stats.as_json().unwrap().contains("statistics"));
    assert!(stats.as_json().unwrap().contains("11")); // requests made
    assert!(stats.as_str().is_empty());
}