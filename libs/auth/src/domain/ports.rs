use crate::{AuthError, Claims, Identity};

pub trait AuthRepository: Send + Sync {
    fn validate_token(&self, token: &str)
    -> impl Future<Output = Result<Claims, AuthError>> + Send;

    fn identify(&self, token: &str) -> impl Future<Output = Result<Identity, AuthError>> + Send;
}

pub trait HasAuthRepository {
    type AuthRepo: AuthRepository;

    fn auth_repository(&self) -> &Self::AuthRepo;
}
