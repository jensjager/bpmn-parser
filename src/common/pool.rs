// lane.rs
use crate::common::lane::Lane;

pub struct Pool {
    pool_name: String,
    lanes: Vec<Lane>,
    pub x: Option<f64>,
    pub y: Option<f64>,
    pub width: Option<f64>,
    pub height: Option<f64>,
}

impl Pool {
    pub fn new(pool_name: String, lanes: Vec<Lane>) -> Self {
        Pool {
            pool_name,
            lanes,
            x: None,
            y: None,
            width: None,
            height: None,
        }
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
