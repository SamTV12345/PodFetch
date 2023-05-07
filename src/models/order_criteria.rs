#[derive(Serialize, Deserialize, Debug,Clone)]
pub enum OrderCriteria {
    ASC,
    DESC
}

impl OrderCriteria{
    pub fn to_bool(self)->bool{
        match self{
            OrderCriteria::ASC=>true,
            OrderCriteria::DESC=>false
        }
    }
}
#[derive(Serialize, Deserialize, Debug,Clone)]
#[serde(rename_all = "UPPERCASE")]
pub enum OrderOption{
    PublishedDate,
    Title
}

impl OrderOption{
    pub fn to_string(self)->String{
        match self{
            OrderOption::PublishedDate=>"PublishedDate".to_string(),
            OrderOption::Title=>"Title".to_string()
        }
    }

    pub fn from_string(s: String)->Self{
        match s.as_str(){
            "PublishedDate"=>OrderOption::PublishedDate,
            "Title"=>OrderOption::Title,
            _=>panic!("Invalid OrderOption")
        }
    }
}
