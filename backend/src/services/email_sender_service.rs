use lettre::{Message, SmtpTransport, Transport};
use lettre::transport::smtp::authentication::Credentials;
use std::env;
use tokio::sync::mpsc::{Sender, Receiver};
use once_cell::sync::OnceCell;

pub struct EmailRequest {
    pub sender: String,
    pub recipient: String,
    pub subject: String,
    pub body: String,
}

/// Global email request queue, initialized only once using OnceCell.
static GLOBAL_EMAIL_QUEUE: OnceCell<Sender<EmailRequest>> = OnceCell::new();

pub async fn enqueue_email(sender: String, recipient: String, subject: String, body: String) {
    // Create a new email request object
    let mail_request = EmailRequest {
        sender,
        recipient,
        subject,
        body,
    };

    // If the global email queue has been initialized, send the request into the queue.
    if let Some(tx) = GLOBAL_EMAIL_QUEUE.get() {
        // Try to send the request to the queue, transmitter send the request
        if tx.send(mail_request).await.is_err() {
            eprintln!("Fail to send email request");
        }
        return;
    }

    eprintln!("Not initialized email service, transmitter does not exist");
}

/// Initializes the email service.
/// Create a 100 size channel for email requests.
/// We have 2 different queues, for resting password and for sending emails.
/// For sending emails queue is bigger.
pub fn init_email_service() {
    let (tx, mut rx): (Sender<EmailRequest>, Receiver<EmailRequest>) = tokio::sync::mpsc::channel(100);

    GLOBAL_EMAIL_QUEUE.set(tx).expect("Email queue already initialized");

    tokio::spawn(async move {
        while let Some(email) = rx.recv().await {
            tokio::task::spawn_blocking(move || {
                let email_msg = Message::builder()
                    .from(format!("Key Manager <{}>", email.sender).parse().unwrap())
                    .to(format!("Receiver <{}>", email.recipient).parse().unwrap())
                    .subject(&email.subject)
                    .body(email.body)
                    .unwrap();

                // Set up the SMTP client
                let mailer = SmtpTransport::relay(env::var("SMTP_SERVER").expect("SMTP_SERVER must be set").as_str())
                    .unwrap()
                    .credentials(Credentials::new(
                        env::var("SMTP_USERNAME").expect("SMTP_USERNAME must be set").to_string(),
                        env::var("SMTP_PASSWORD").expect("SMTP_PASSWORD must be set").to_string()
                    ))
                    .build();

                // Send the email
                match mailer.send(&email_msg ) {
                    Ok(_) => println!("Email {} sent successfully!", email.subject),
                    Err(e) => eprintln!("Error sending email: {:?}", e),
                }
            });
        }
    });
}
