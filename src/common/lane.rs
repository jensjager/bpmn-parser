//lane.rs
use crate::common::layer::Layer;

pub struct Lane {
    lane: String,
    layers: Vec<Layer>,
    lanes: Vec<Lane>,
    pub x: Option<f64>,
    pub y: Option<f64>,
    pub width: Option<f64>,
    pub height: Option<f64>,
}

impl Lane {
    pub fn new(lane: String, layers: Vec<Layer>) -> Self {
        Lane {
            lane,
            layers,
            lanes: Vec::new(),
            x: None,
            y: None,
            width: None,
            height: None,
        }
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
