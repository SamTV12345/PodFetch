use std::fmt;
use std::fmt::{Display, Formatter};
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub enum Color{
    Red,
    Green,
    Blue,
}

impl Display for Color {
        fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
            match self {
                Color::Red => write!(f, "Red"),
                Color::Green => write!(f, "Green"),
                Color::Blue => write!(f, "Blue"),
            }
        }
}