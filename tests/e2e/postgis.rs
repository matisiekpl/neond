use crate::common::{api, connect, exclusive};

#[tokio::test]
async fn postgis_is_available_on_compute() {
    let _guard = exclusive().await;
    let api = api().await;

    let organization_id = api.create_organization("PostGIS Org").await;
    let project_id = api.create_project(&organization_id, "PostGIS Project").await;
    let branch = api
        .create_branch(&organization_id, &project_id, "spatial", None)
        .await;
    let branch_id = branch["id"].as_str().expect("branch id missing");
    let endpoint = api
        .start_endpoint(&organization_id, &project_id, branch_id)
        .await;
    let client = connect(&endpoint).await;

    let available: bool = client
        .query_one(
            "SELECT EXISTS (SELECT 1 FROM pg_available_extensions WHERE name = 'postgis')",
            &[],
        )
        .await
        .expect("failed to list available extensions")
        .get(0);
    assert!(available, "PostGIS is missing from the compute PostgreSQL installation");

    // Use the same postgres credentials exposed to users, not cloud_admin.
    client
        .batch_execute("CREATE EXTENSION postgis")
        .await
        .expect("compute database owner could not enable PostGIS");

    let version: String = client
        .query_one("SELECT PostGIS_Full_Version()", &[])
        .await
        .expect("PostGIS could not load its runtime libraries")
        .get(0);
    assert!(version.contains("POSTGIS="), "unexpected PostGIS version: {version}");

    // Buffer exercises GEOS; reprojection exercises PROJ and its coordinate-system data.
    let row = client
        .query_one(
            "SELECT ST_Area(ST_Buffer(ST_MakePoint(0, 0), 1)),
                    ST_X(ST_Transform(ST_SetSRID(ST_MakePoint(1, 0), 4326), 3857))",
            &[],
        )
        .await
        .expect("PostGIS geometry or coordinate transformation failed");
    let area: f64 = row.get(0);
    let projected_x: f64 = row.get(1);
    assert!((3.0..3.2).contains(&area), "unexpected unit buffer area: {area}");
    assert!(
        (projected_x - 111_319.490_793).abs() < 0.01,
        "unexpected EPSG:3857 coordinate: {projected_x}"
    );

    drop(client);
    api.stop_endpoint(&organization_id, &project_id, branch_id).await;
}
