// pool.rs
use crate::common::graph::NodeId;
use crate::common::lane::Lane;
pub struct Pool {
    pub pool_name: Option<String>,
    pub lanes: Vec<Lane>,
    pub x: Option<f64>,
    pub y: Option<f64>,
    pub width: Option<f64>,
    pub height: Option<f64>,
}

impl Pool {
    pub fn new(pool_name: Option<String>) -> Self {
        Pool {
            pool_name,
            lanes: Vec::new(),
            x: None,
            y: None,
            width: None,
            height: None,
        }
    }

    pub fn add_node(&mut self, lane: Option<String>, node_id: NodeId) {
        // TODO clean up
        if let Some(lane) = self.lanes.iter_mut().find(|l| l.lane == lane) {
            lane.nodes.push(node_id);
        } else {
            let mut new_lane = Lane::new(lane);
            new_lane.nodes.push(node_id);
            self.lanes.push(new_lane);
        }
    }

    //    pub fn set_height(&mut self, height: f64) {
    //        self.height = Some(height);
    //    }
    //
    //    pub fn set_width(&mut self, width: f64) {
    //        self.width = Some(width);
    //    }
    //
    //    pub fn set_position(&mut self, x: f64, y: f64) {
    //        self.x = Some(x);
    //        self.y = Some(y);
    //    }
}
