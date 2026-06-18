#[derive(Debug, Clone)]
pub struct CreateUserCommand {
    pub name: String,
    pub username: String,
    pub email: String,
    pub sub: String,
}

#[derive(Debug, Clone)]
pub struct UpsertUserBySubCommand {
    pub sub: String,
    pub email: String,
    pub username: String,
    pub name: Option<String>,
}
