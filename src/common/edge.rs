#[derive(Debug, Clone)]
pub struct Edge {
    pub from: usize,
    pub to: usize,
    pub text: Option<String>,
    pub bend_points: Vec<(f64, f64)>, // Muuda õigeks, et see oleks (f64, f64) paaride vektor
}

impl Edge {
    pub fn new(from: usize, to: usize, text: Option<String>) -> Self {
        Edge {
            from,
            to,
            text,
            bend_points: vec![], // Alguses tühjad painutuspunktid
        }
    }

    // pub fn with_default_text(from: usize, to: usize) -> Self {
    //     Edge {
    //         from,
    //         to,
    //         text: None,
    //         bend_points: vec![], // Alguses tühjad painutuspunktid
    //     }
    // }
}
