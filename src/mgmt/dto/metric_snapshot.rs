use std::collections::HashMap;
use uuid::Uuid;

#[derive(Clone, Eq, PartialEq, Hash)]
pub struct LatestMetricKey {
    pub slug: String,
    pub branch_id: Option<Uuid>,
}

#[derive(Clone)]
pub struct BranchLabels {
    pub branch_name: String,
    pub project_name: String,
}

pub type LatestMetrics = HashMap<LatestMetricKey, f64>;
pub type BranchLabelMap = HashMap<Uuid, BranchLabels>;

#[derive(Clone, Default)]
pub struct MetricSnapshot {
    pub samples: LatestMetrics,
    pub branch_labels: BranchLabelMap,
}