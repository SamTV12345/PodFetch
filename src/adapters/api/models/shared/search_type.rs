pub enum SearchType {
    ITunes,
    Podindex,
}

impl TryFrom<i32> for SearchType {
    type Error = ();

    fn try_from(v: i32) -> Result<Self, Self::Error> {
        match v {
            x if x == SearchType::Podindex as i32 => Ok(SearchType::Podindex),
            x if x == SearchType::ITunes as i32 => Ok(SearchType::ITunes),
            _ => Err(()),
        }
    }
}
