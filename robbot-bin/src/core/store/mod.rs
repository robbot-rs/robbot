#[derive(Default)]
pub struct Store {
    pub pool: Option<sqlx::MySqlPool>,
}
