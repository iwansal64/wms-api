use std::env;

use crate::util::{generate_token, is_duplicated_error};
use lettre::{
    Message, Transport,
    transport::smtp::{
        SmtpTransport,
        authentication::{Credentials, Mechanism},
    },
};
use rocket::{State, http::Status, post, serde::json::Json};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct RegistrationRequestType {
    email: String,
}

#[post("/user/register/1", data = "<registration_data>")]
pub async fn post(
    registration_data: Json<RegistrationRequestType>,
    db: &State<sqlx::postgres::PgPool>,
) -> Result<(), Status> {
    // Generate verification token and user id
    let generated_verification_token: String = generate_token();
    let generated_id: String = generate_token();


    // Setting up email message
    let mail: Message = Message::builder()
        // From which email account?
        .from(
            (env::var("EMAIL_APP_ACCOUNT")
                .expect("Set your email app account up in the environment bruh")
                + "+gaia-support")
                .parse()
                .expect("There's an error when trying to parse the 'from' email account"),
        )
        // Send to where?
        .to(registration_data
            .email
            .parse()
            .expect("There's an error when trying to parse the 'to' email account"))
        // Subject
        .subject("Email Verification Process")
        // Header
        .header(lettre::message::header::ContentType::TEXT_PLAIN)
        // body
        .body(format!(
            "Here's the token for verification: {}",
            generated_verification_token
        ))
        // Check if there's an error
        .expect("There's an error when building email message");


    // Setting up email sender
    let mailer: SmtpTransport = SmtpTransport::starttls_relay("smtp.gmail.com")
        // If there's an error
        .expect("There's an error when trying to start SMTP TLS relay")
        // Put the credentials
        .credentials(Credentials::new(
            env::var("EMAIL_APP_ACCOUNT").unwrap(),
            env::var("EMAIL_APP_PASSWORD")
                .expect("Set your email app password up in the environment bruh"), // Use an app password, not your account password
        ))
        // Set the authentication mechanism
        .authentication(vec![Mechanism::Plain])
        // Build the SMTP TLS relay
        .build();
    

    // Send an email verification to the given email
    let result = mailer.send(&mail);

    match result {
        Ok(_) => (),
        Err(err) => {
            log::error!(
                "There's an error when trying to send email. Error: {}",
                err.to_string()
            );
            return Err(Status::InternalServerError);
        }
    }

    // Create a user data with verification token
    {
        let raw_user_data = sqlx::query!(
            "INSERT INTO users(id, email, verification_token) VALUES ($1, $2, $3)",
            generated_id,
            registration_data.email,
            generated_verification_token
        )
        .execute(db.inner())
        .await;

        match raw_user_data {
            Ok(_) => (),
            Err(err) => {
                if is_duplicated_error(&err) {
                    return Err(Status::Conflict);
                }

                log::error!(
                    "There's an error when trying to register user. Error: {}",
                    err.to_string()
                );
                return Err(Status::InternalServerError);
            }
        }
    }
    
    
    // Send the OK result
    Ok(())
}
