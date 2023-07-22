use std::fmt::{Display, Formatter};

#[derive(Serialize, Deserialize, Debug,Clone)]
pub enum OrderCriteria {
    #[serde(rename = "ASC")]
    Asc,
    #[serde(rename = "DESC")]
    Desc
}

impl OrderCriteria{
    pub fn to_bool(&self)->bool{
        match self{
            OrderCriteria::Asc =>true,
            OrderCriteria::Desc =>false
        }
    }
}
#[derive(Serialize, Deserialize, Debug,Clone)]
#[serde(rename_all = "UPPERCASE")]
pub enum OrderOption{
    PublishedDate,
    Title
}

impl Display for OrderOption{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self{
            OrderOption::PublishedDate=>write!(f,"PublishedDate"),
            OrderOption::Title=>write!(f,"Title")
        }
    }
}
impl OrderOption{
    pub fn from_string(s: String)->Self{
        match s.as_str(){
            "PublishedDate"=>OrderOption::PublishedDate,
            "Title"=>OrderOption::Title,
            _=>panic!("Invalid OrderOption")
        }
    }
}
