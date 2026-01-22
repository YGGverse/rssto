use rocket::serde::Serialize;

#[derive(Clone, Debug, Serialize)]
#[serde(crate = "rocket::serde")]
pub struct Meta {
    pub description: Option<String>,
    pub title: String,
    pub version: String,
}
