#[derive(Debug, Clone)]
pub struct Edge {
    pub from: usize,
    pub to: usize,
    pub text: Option<String>,
    pub adjusted_points: Option<Vec<(f64, f64)>>, // Uued, lõplikud punktid, mis hõlmavad algus-, lõpp- ja painutuspunkte
}

impl Edge {
    pub fn new(from: usize, to: usize, text: Option<String>) -> Self {
        Edge {
            from,
            to,
            text,
            adjusted_points: None, // Alguses tühi, määratakse assign_bend_points-s
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
