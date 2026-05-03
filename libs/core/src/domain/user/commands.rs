#[derive(Debug, Clone)]
pub struct CreateUserCommand {
    pub name: String,
    pub username: String,
    pub email: String,
    pub sub: String,
}
