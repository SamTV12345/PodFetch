#[derive(Serialize, Deserialize, Debug)]
pub enum OrderCriteria {
    ASC,
    DESC
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "UPPERCASE")]
pub enum OrderOption{
    PublishedDate,
    Title
}