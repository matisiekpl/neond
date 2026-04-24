use chrono::NaiveDateTime;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use crate::mgmt::dto::error::{AppError, Result};
use crate::mgmt::model::compute_metric_sample::{ComputeMetricSample, NewComputeMetricSample};
use crate::mgmt::repository::db::DbPool;
use crate::mgmt::schema::schema::compute_metric_samples;

#[derive(Clone)]
pub struct MetricRepository {
    pool: DbPool,
}

impl MetricRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    pub async fn insert_batch(&self, samples: Vec<NewComputeMetricSample>) -> Result<()> {
        if samples.is_empty() {
            return Ok(());
        }
        let connection = &mut self
            .pool
            .get()
            .await
            .map_err(|error| AppError::Internal(error.to_string()))?;
        diesel::insert_into(compute_metric_samples::table)
            .values(&samples)
            .execute(connection)
            .await
            .map_err(AppError::from)?;
        Ok(())
    }

    pub async fn list_for_branch(
        &self,
        branch_id: Uuid,
        from: NaiveDateTime,
        to: NaiveDateTime,
    ) -> Result<Vec<ComputeMetricSample>> {
        let connection = &mut self
            .pool
            .get()
            .await
            .map_err(|error| AppError::Internal(error.to_string()))?;
        compute_metric_samples::table
            .filter(compute_metric_samples::branch_id.eq(branch_id))
            .filter(compute_metric_samples::recorded_at.ge(from))
            .filter(compute_metric_samples::recorded_at.le(to))
            .order(compute_metric_samples::recorded_at.asc())
            .load::<ComputeMetricSample>(connection)
            .await
            .map_err(Into::into)
    }

    pub async fn delete_older_than(&self, cutoff: NaiveDateTime) -> Result<usize> {
        let connection = &mut self
            .pool
            .get()
            .await
            .map_err(|error| AppError::Internal(error.to_string()))?;
        let deleted = diesel::delete(
            compute_metric_samples::table
                .filter(compute_metric_samples::recorded_at.lt(cutoff)),
        )
        .execute(connection)
        .await
        .map_err(|error| AppError::Internal(error.to_string()))?;
        Ok(deleted)
    }
}
