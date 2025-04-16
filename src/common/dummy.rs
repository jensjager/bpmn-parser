#[derive(Debug, Clone, Default)]
pub struct Dummy {
    pub is_data: bool,
    pub is_reversed: bool,
    pub from: Option<usize>,
    pub to: Option<usize>,
    pub skip: bool,
}
