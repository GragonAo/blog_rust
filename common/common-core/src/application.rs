use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Snowflake {
    pub machine_id: i32,
    pub node_id: i32,
}
