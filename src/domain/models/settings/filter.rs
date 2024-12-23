pub struct Filter {
    pub username: String,
    pub title: Option<String>,
    pub ascending: bool,
    pub filter: Option<String>,
    pub only_favored: bool,
}

impl Filter {
    pub(crate) fn new(username: String, title: Option<String>, ascending: bool, filter:
    Option<String>, only_favored: bool) -> Self{
        Filter {
            username,
            title,
            ascending,
            filter,
            only_favored,
        }
    }
}