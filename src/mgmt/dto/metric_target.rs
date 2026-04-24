use uuid::Uuid;

#[derive(Clone, Debug)]
pub struct MetricTarget {
    pub branch_id: Uuid,
    pub pid: u32,
    pub pg_port: u16,
    pub metrics_port: u16,
}
