use axum::{
    http::{header, StatusCode},
    extract::State, response::IntoResponse, Json
};
// use lettre::{
//     message::{header::ContentType, Mailbox, Message}, Address, AsyncTransport
// };
use reqwest::Client;
use serde::Deserialize;
use serde_json::json;
// use std::sync::Arc;
use serde::Serialize;
use tower_sessions::Session;
use crate::utils::data::BODY;

use crate::{error::AppError, utils::get_new_uuid_v4, AppState};


#[derive(Serialize)]
struct EmailAddress {
    email: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
}

#[derive(Serialize)]
struct MailtrapPayload {
    from: EmailAddress,
    to: Vec<EmailAddress>,
    subject: String,
    html: String,
    category: String,
}


#[derive(Deserialize)]
pub struct EmailRequest {
    pub to: String,
    pub subject: String
}

pub async fn send_email(
    session: Session,
    State(state): State<AppState>,
    Json(payload): Json<EmailRequest>,
) -> Result<impl IntoResponse, AppError> {
    let api_token = "478d51b65372f0f165f433f52bf15321";
    let sandbox_id = "3804420";

    let new_uuid_v4 = get_new_uuid_v4();
    let body = BODY.replace("{{RESET_LINK}}", &format!(r#"http://localhost:3000/resetPassword/{new_uuid_v4}"#));
    let client = Client::new();
    let email_payload = MailtrapPayload {
        from: EmailAddress {
            email: "help@loony.com".to_string(),
            name: Some("Mailtrap Test".to_string()),
        },
        to: vec![EmailAddress {
            email: payload.to,
            name: None,
        }],
        subject: payload.subject,
        html: body,
        category: "Integration Test".to_string(),
    };

    let _ = client
        .post(&format!(
            "https://sandbox.api.mailtrap.io/api/send/{}",
            sandbox_id
        ))
        .bearer_auth(api_token)
        .json(&email_payload)
        .send()
        .await;

    session
        .insert("RESET_PASSWORD_SESSION_ID", new_uuid_v4)
        .await?;

    Ok((
        StatusCode::OK,
        [(header::CONTENT_TYPE, "application/json")],
        Json(json!({
            "status": 200,
            "message": "Mail sent."
        })),
    ))
}