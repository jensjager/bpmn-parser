//lane.rs
use crate::common::layer::Layer;

pub struct Lane {
    lane: String,
    layers: Vec<Layer>,
}

impl Lane {
    pub fn new(lane: String, layers: Vec<Layer>) -> Self {
        Lane { lane, layers }
    }

    pub fn get_layers(&self) -> &Vec<Layer> {
        &self.layers
    }

    pub fn get_layers_mut(&mut self) -> &mut Vec<Layer> {
        &mut self.layers
    }

    pub fn get_lane(&self) -> &String {
        &self.lane
    }
}
