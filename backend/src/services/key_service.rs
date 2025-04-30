
// rotation_service
/// TODO: Implement tools for generation or encryption passwords, rotation.

use tokio_cron_scheduler::{JobScheduler, Job};
use std::env;
use std::error::Error;
use std::time::Duration;
use chrono::NaiveDateTime;
use crate::services::email_sender_service::{enqueue_email};

use sqlx::{FromRow, PgPool};

# [derive(FromRow)]
pub struct KeyRelevanceService {
    user_id: i32,
    email: String,
    key_name: String,
    key_type: String,
    key_description: String,
    created_at: NaiveDateTime,
    expiration_date: NaiveDateTime,
}

pub async fn check_keys_relevance(pool: &PgPool) -> Result<(), Box<dyn Error>> {
    let sched = JobScheduler::new().await?;

    // Run every Saturday at 12:00(env).
    let scheduler_run_time = env::var("SCHEDULER_RUN_TIME").unwrap_or_else(|_| "0 * * * * *".to_string());

    let job = Job::new_async(scheduler_run_time, {
        let pool = pool.clone();

        // Use closure with task id and locker scheduler
        move |_uuid, _l| {
            let pool = pool.clone();

            Box::pin(async move {
                // Retrieve expired keys from the database
                let expired_keys = match sqlx::query_as::<_, KeyRelevanceService>(
                    "SELECT
                            users.id AS user_id, users.email, keys.key_name,
                            keys.key_type, keys.key_description, keys.created_at,
                            keys.expiration_date
                         FROM keys
                         JOIN users ON keys.user_id = users.id
                         WHERE keys.expiration_date >= NOW() - INTERVAL '7 days' AND keys.expiration_date < NOW()
                            OR keys.expiration_date >= NOW()"
                )
                .fetch_all(&pool)
                .await {
                    Ok(keys) => keys,
                    Err(e) => {
                        eprintln!("Database error: {:?}", e);
                        return;
                    }
                };

                // Create email notification
                let smtp_username = env::var("SMTP_USERNAME").expect("SMTP_USERNAME must be set");

                // Loop for sending emails
                for key in expired_keys {
                    let user_id = key.user_id;
                    let recipient_email = key.email.clone();
                    let key_name = key.key_name.clone();
                    let key_type = key.key_type.clone();
                    let key_description = key.key_description.clone();
                    let created_at = key.created_at.clone();
                    let expiration_date = key.expiration_date.clone();

                    // Create email body
                    let email_body = format!(
                        "Your key '{}'.\n\
                         Type '{}' \n\
                         Description: '{}' \n\
                         Created on {} has expired on {}. \n\n\
                         Please take action to renew or replace it.",
                        key_name, key_type, key_description, created_at.format("%Y-%m-%d %H:%M:%S"), expiration_date.format("%Y-%m-%d %H:%M:%S")
                    );

                    // Enqueue email for sending
                    enqueue_email(
                        smtp_username.clone(),
                        recipient_email.clone(),
                        "Key Expiration Notification⚠️".to_string(),
                        email_body
                    ).await;

                    // Try to create a notification in the database
                    let message;
                    if expiration_date > chrono::Utc::now().naive_utc(){
                        message = format!("Key '{}' expired", key_name);
                    } else {
                        message = format!("Key '{}' is about to expire", key_name);
                    }
                    if let Err(e) = sqlx::query("INSERT INTO notifications (
                        user_id,
                        notification_type,
                        message
                    ) VALUES ($1, $2, $3)")
                    .bind(user_id)
                    .bind("key_expiration")
                    .bind(message)
                    .execute(&pool)
                    .await {
                        eprintln!("Database error while inserting notification: {:?}", e);
                    }

                    // Sleep 500 milliseconds between sending next email
                    tokio::time::sleep(Duration::from_millis(500)).await;
                }
            })
        }
    })?;

    sched.add(job).await?;
    sched.start().await?;

    // Keep the scheduler running indefinitely. Sleep for 1 hour.
    loop { tokio::time::sleep(Duration::from_secs(3600)).await; }
}