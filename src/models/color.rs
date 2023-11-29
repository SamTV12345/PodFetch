use std::fmt;
use std::fmt::{Display, Formatter};

#[derive(Debug, Serialize, Deserialize)]
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