pub enum SearchType {
    ITUNES,
    PODINDEX,
}

impl TryFrom<i32> for SearchType {
    type Error = ();

    fn try_from(v: i32) -> Result<Self, Self::Error> {
        match v {
            x if x == SearchType::PODINDEX as i32 => Ok(SearchType::PODINDEX),
            x if x == SearchType::ITUNES as i32 => Ok(SearchType::ITUNES),
            _ => Err(()),
        }
    }
}
