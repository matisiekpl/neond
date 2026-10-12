use chrono::Utc;
use std::time::Duration;

use crate::common::{api, checksum, connect, exclusive, safekeeper_wal_segments, seed, wait_for_ingest};

const WAL_SEGMENT_BYTES: i64 = 16 * 1024 * 1024;
const SEED_MEGABYTES: i64 = 400;

#[tokio::test]
async fn safekeeper_removes_wal_once_pageserver_has_it() {
    let _guard = exclusive().await;
    let api = api().await;

    let organization_id = api.create_organization("Wal Retention Org").await;
    let project_id = api.create_project(&organization_id, "Wal Retention Project").await;

    let branch = api
        .create_branch(&organization_id, &project_id, "production", None)
        .await;
    let branch_id = branch["id"].as_str().unwrap().to_string();
    let timeline_id = branch["timeline_id"].as_str().unwrap().to_string();
    let endpoint = api.start_endpoint(&organization_id, &project_id, &branch_id).await;
    let client = connect(&endpoint).await;

    seed(&client, 50).await;
    wait_for_ingest(&api, &organization_id, &project_id, &branch_id, &client).await;
    let checksum_at_recovery_point = checksum(&client).await;
    let recovery_point = Utc::now();

    tokio::time::sleep(Duration::from_secs(10)).await;

    seed(&client, SEED_MEGABYTES).await;
    wait_for_ingest(&api, &organization_id, &project_id, &branch_id, &client).await;
    let checksum_after_load = checksum(&client).await;

    let written_bytes: i64 = client
        .query_one("SELECT (pg_current_wal_flush_lsn() - '0/0'::pg_lsn)::bigint", &[])
        .await
        .expect("failed to read flush lsn")
        .get(0);
    let written_segments = written_bytes / WAL_SEGMENT_BYTES;
    assert!(
        written_segments >= 20,
        "expected the load to write at least 20 WAL segments, wrote {written_segments}"
    );

    // Without WAL backup the safekeeper used to keep every segment since the timeline was
    // created. Once the pageserver has checkpointed and uploaded the load (checkpoint_timeout)
    // and the safekeeper has persisted that in its control file, only the tail must remain.
    let deadline = std::time::Instant::now() + Duration::from_secs(900);
    loop {
        let segments = safekeeper_wal_segments(&project_id, &timeline_id);
        println!("[wal] safekeeper keeps {segments} of {written_segments} written WAL segments");
        if segments <= 4 {
            break;
        }
        assert!(
            std::time::Instant::now() < deadline,
            "safekeeper still keeps {segments} of {written_segments} written WAL segments"
        );
        tokio::time::sleep(Duration::from_secs(15)).await;
    }

    drop(client);
    api.stop_endpoint(&organization_id, &project_id, &branch_id).await;
    let client = connect(&api.start_endpoint(&organization_id, &project_id, &branch_id).await).await;
    assert_eq!(
        checksum(&client).await,
        checksum_after_load,
        "data must survive removal of the safekeeper's WAL"
    );
    drop(client);

    let lsn = api
        .lsn_at(&organization_id, &project_id, &branch_id, recovery_point)
        .await;
    let (status, restored) = api.restore(&organization_id, &project_id, &branch_id, &lsn).await;
    assert!(status.is_success(), "restore failed with {status}: {restored}");
    let branch_id = restored["id"].as_str().unwrap().to_string();
    let client = connect(&api.endpoint(&organization_id, &project_id, &branch_id).await).await;
    assert_eq!(
        checksum(&client).await,
        checksum_at_recovery_point,
        "point-in-time restore from before the removed WAL must still work"
    );
}
