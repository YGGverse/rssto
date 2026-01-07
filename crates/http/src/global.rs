use rocket::serde::Serialize;

#[derive(Clone, Debug, Serialize)]
#[serde(crate = "rocket::serde")]
pub struct Global {
    pub list_limit: usize,
    pub format_time: String,
}
