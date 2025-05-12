use std::collections::HashMap;
use tokio_cron_scheduler::{JobScheduler, Job};
use std::env;
use std::error::Error;
use std::time::Duration;
use chrono::{Local, NaiveDateTime};
use crate::services::{enqueue_email, EmailRequest, GLOBAL_SENDER_EMAIL};
use sqlx::{FromRow, PgPool};

#[derive(FromRow)]
pub struct UserKey {
    user_id: i32,
    email: String,
    key_name: String,
    expiration_date: Option<NaiveDateTime>,
}

pub async fn check_keys_relevance(pool: PgPool) -> Result<JobScheduler, Box<dyn Error>> {
    let schedule = JobScheduler::new().await?;

    let scheduler_run_time = "0 * * * * *".to_string();

    schedule.add(Job::new_async(&scheduler_run_time, move |_uuid, _lock| {
        let pool = pool.clone();
        
        Box::pin(async move {
            println!("Scheduler is running at: {}", chrono::Local::now());
            
            match check_and_notify_keys(&pool).await {
                Ok(_) => println!("Key relevance check completed successfully."),
                Err(e) => eprintln!("Error during key relevance check."),
            }
        })
    })?).await?;

    schedule.start().await?;
    Ok(schedule)
}

async fn check_and_notify_keys(pool: &PgPool) -> Result<(), Box<dyn Error>> {
    let keys = sqlx::query_as!(
        UserKey,
        "SELECT users.id AS user_id, users.email, keys.key_name, keys.expiration_date
         FROM keys
         JOIN users ON keys.user_id = users.id
         WHERE keys.expiration_date <= NOW() + INTERVAL '2 days'
         ORDER BY users.id"
    )
    .fetch_all(pool)
    .await?;

    let mut user_keys: HashMap<i32, Vec<&UserKey>> = HashMap::new();
    for key in &keys {
        user_keys.entry(key.user_id)
            .or_insert_with(Vec::new)
            .push(key);
    }

    let date = Local::now().naive_local();
    
    for (_user_id, user_keys) in user_keys {
        let email = &user_keys[0].email;

        let mut expired_keys = Vec::new();
        let mut about_to_expire_keys = Vec::new();

        for key in user_keys {
            if key.expiration_date.unwrap() <= date {
                expired_keys.push(key);
            } else {
                about_to_expire_keys.push(key);
            }
        }

        let mut email_body = String::new();

        if !expired_keys.is_empty() {
            email_body.push_str("Expired Keys:\n");
            for key in expired_keys {
                email_body.push_str(&format!(
                    "- Name: {} Expired on: {}\n",
                    key.key_name,
                    key.expiration_date.unwrap()
                        .format("%Y-%m-%d %H:%M:%S")
                ));
            }
        }

        if !about_to_expire_keys.is_empty() {
            if !email_body.is_empty() {
                email_body.push_str("\n");
            }

            email_body.push_str("Keys About to Expire:\n");
            for key in about_to_expire_keys {
                email_body.push_str(&format!(
                    "- Name: {} Expires on: {}\n",
                    key.key_name,
                    key.expiration_date.unwrap()
                        .format("%Y-%m-%d %H:%M:%S")
                ));
            }
        }

        email_body.push_str("\nPlease take action to renew or replace these keys.");

        let email_request = EmailRequest {
            sender: GLOBAL_SENDER_EMAIL.to_string(),
            recipient: email.clone(),
            subject: "Key Expiration Notification⚠️".to_string(),
            body: email_body,
        };

        enqueue_email(email_request).await;

        tokio::time::sleep(Duration::from_millis(500)).await;
    }

    Ok(())
}

pub async fn init_scheduler(pool: PgPool) {
    check_keys_relevance(pool).await
        .expect("Failed to initialize scheduler");
}
