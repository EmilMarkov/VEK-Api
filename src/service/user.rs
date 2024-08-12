use crate::prisma::{user, PrismaClient};
use chrono::Utc;
use prisma_client_rust::QueryError;

pub struct UserService;

impl UserService {
    pub async fn get_user_by_id(
        client: &PrismaClient,
        id: &str,
    ) -> Result<Option<user::Data>, QueryError> {
        client
            .user()
            .find_first(vec![user::id::equals(id.to_string())])
            .exec()
            .await
    }

    pub async fn get_user_by_login(
        client: &PrismaClient,
        login: &str,
    ) -> Result<Option<user::Data>, QueryError> {
        client
            .user()
            .find_first(vec![user::login::equals(login.to_string())])
            .exec()
            .await
    }

    pub async fn update_user_timestamp(
        client: &PrismaClient,
        user_id: &str,
    ) -> Result<user::Data, QueryError> {
        client
            .user()
            .update(
                user::id::equals(user_id.to_string()),
                vec![user::updated_at::set(Utc::now().into())],
            )
            .exec()
            .await
    }

    pub async fn create_user(
        client: &PrismaClient,
        login: &str,
    ) -> Result<user::Data, QueryError> {
        client
            .user()
            .create(login.to_string(), vec![])
            .exec()
            .await
    }
}
