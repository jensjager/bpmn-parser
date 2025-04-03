use crate::common::graph::NodeId;

#[derive(Debug, Clone, Default)]
pub struct Edge {
    pub from: NodeId,
    pub to: NodeId,
    pub is_dummy: bool,
    pub temp_disabled: bool,
    pub text: Option<String>,
    pub bend_points: Option<Vec<(f64, f64)>>, // Uued, lõplikud punktid, mis hõlmavad algus-, lõpp- ja painutuspunkte
}

impl Edge {
    // pub fn with_default_text(from: usize, to: usize) -> Self {
    //     Edge {
    //         from,
    //         to,
    //         text: None,
    //         bend_points: vec![], // Alguses tühjad painutuspunktid
    //     }
    // }
}
