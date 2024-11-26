// lane.rs
use crate::common::lane::Lane;

pub struct Pool {
    pool_name: String,
    lanes: Vec<Lane>,
}

impl Pool {
    pub fn new(pool_name: String, lanes: Vec<Lane>) -> Self {
        Pool { pool_name, lanes }
    }

    pub fn add_lane(&mut self, lane: Lane) {
        self.lanes.push(lane);
    }

    pub fn get_lanes(&self) -> &Vec<Lane> {
        &self.lanes
    }

    pub fn get_lanes_mut(&mut self) -> &mut Vec<Lane> {
        &mut self.lanes
    }

    pub fn get_pool_name(&self) -> &String {
        &self.pool_name
    }
}
