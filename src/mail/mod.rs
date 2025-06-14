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
    let mailtrap = &state.mailtrap;

    let new_uuid_v4 = get_new_uuid_v4();
    let body = BODY.replace("{{RESET_LINK}}", &format!(r#"http://localhost:3000/resetPassword/{new_uuid_v4}"#));
    let client = Client::new();
    let email_payload = MailtrapPayload {
        from: EmailAddress {
            email: mailtrap.mailtrap_email.clone(),
            name: mailtrap.mailtrap_name.clone(),
        },
        to: vec![EmailAddress {
            email: payload.to,
            name: None,
        }],
        subject: payload.subject,
        html: body,
        category: "Integration Test".to_string(),
    };

    let url = &mailtrap.url;

    let _ = client
        .post(url)
        .bearer_auth(&mailtrap.mailtrap_token_id)
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