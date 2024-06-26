// src/routes/users/confirm_registration.rs
#[derive(serde::Deserialize)]
pub struct Parameters {
    token: String,
}

#[tracing::instrument(
    name = "Activating a new user",
    skip(pool, parameters, redis_pool)
)]
#[actix_web::get("/register/confirm")]
pub async fn confirm(
    parameters: actix_web::web::Query<Parameters>,
    pool: actix_web::web::Data<sqlx::postgres::PgPool>,
    redis_pool: actix_web::web::Data<deadpool_redis::Pool>,
) -> actix_web::HttpResponse {
    let settings = crate::settings::get_settings().expect("Failed to read settings.");

    let mut redis_con = redis_pool
        .get()
        .await
        .map_err(|e| {
            tracing::event!(target: "backend", tracing::Level::ERROR, "{}", e);

            actix_web::HttpResponse::SeeOther()
                .insert_header((
                    actix_web::http::header::LOCATION,
                    format!("{}/auth/error", settings.frontend_url),
                ))
                .json(crate::types::ErrorResponse {
                    error: "We cannot activate your account at the moment".to_string(),
                })
        })
        .expect("Redis connection cannot be obtained.");

    let confirmation_token = match crate::utils::verify_confirmation_token_pasetor(
        parameters.token.clone(),
        &mut redis_con,
        None
    )
    .await
    {
        Ok(token) => token,
        Err(e) => {
            tracing::event!(target: "backend", tracing::Level::ERROR, "{:#?}", e);

            return actix_web::HttpResponse::SeeOther().insert_header((
                    actix_web::http::header::LOCATION,
                    format!("{}/auth/regenerate-token", settings.frontend_url),
                )).json(crate::types::ErrorResponse {
                    error: "It appears that your confirmation token has expired or previously used. Please generate a new token".to_string(),
                });
        }
    };
    match activate_new_user(&pool, confirmation_token.user_id).await {
        Ok(_) => {
            tracing::event!(target: "backend", tracing::Level::INFO, "New user was activated successfully.");

            actix_web::HttpResponse::SeeOther()
                .insert_header((
                    actix_web::http::header::LOCATION,
                    format!("{}/auth/confirmed", settings.frontend_url),
                ))
                .json(crate::types::SuccessResponse {
                    message:
                        "Your account has been activated successfully!!! You can now log in"
                            .to_string(),
                })
        }
        Err(e) => {
            tracing::event!(target: "backend", tracing::Level::ERROR, "Cannot activate account: {}", e);

            actix_web::HttpResponse::SeeOther()
                .insert_header((
                    actix_web::http::header::LOCATION,
                    format!("{}/auth/error?reason={e}", settings.frontend_url),
                ))
                .json(crate::types::ErrorResponse {
                    error: "We cannot activate your account at the moment".to_string(),
                })
        }
    }
}

#[tracing::instrument(name = "Mark a user active", skip(pool), fields(
    new_user_user_id = %user_id
))]
pub async fn activate_new_user(
    pool: &sqlx::postgres::PgPool,
    user_id: uuid::Uuid,
) -> Result<(), sqlx::Error> {
    match sqlx::query("UPDATE users SET is_active=true WHERE id = $1")
        .bind(user_id)
        .execute(pool)
        .await
    {
        Ok(_) => Ok(()),
        Err(e) => {
            tracing::error!("Failed to execute query: {:#?}", e);
            Err(e)
        }
    }
}