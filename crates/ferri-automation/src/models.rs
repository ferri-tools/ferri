use std::collections::HashMap;

pub struct Workload {
    pub command: String,
    pub env: Option<HashMap<String, String>>,
}