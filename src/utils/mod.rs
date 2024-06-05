use crate::error::AppError;
use tower_sessions::Session;

pub trait GetUserId {
    async fn get_user_id(&self) -> Result<i32, AppError>;
}

impl GetUserId for Session {
    async fn get_user_id(&self) -> Result<i32, AppError> {
        let user_id: i32 = match self.get("AUTH_USER_ID").await {
            Ok(x) => match x {
                Some(x) => x,
                None => {
                    return Err(AppError::InternalServerError(
                        "User session not found".to_string(),
                    ))
                }
            },
            Err(e) => return Err(AppError::InternalServerError(e.to_string())),
        };
        Ok(user_id)
    }
}
