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

pub const GLOBAL_SENDER_EMAIL: &str = "noreply@key-manager.com";

static GLOBAL_EMAIL_QUEUE: OnceCell<Sender<EmailRequest>> = OnceCell::new();

pub async fn enqueue_email(email: EmailRequest) {
    
    if let Some(tx) = GLOBAL_EMAIL_QUEUE.get() {
        if tx.send(email).await.is_err() {
            eprintln!("Failed to send an email request.");
        }
        return;
    }

    eprintln!("Not initialized email service, transmitter does not exist.");
}

pub fn init_email_service() {

    let (tx, mut rx): (Sender<EmailRequest>, Receiver<EmailRequest>) = tokio::sync::mpsc::channel(20);

    GLOBAL_EMAIL_QUEUE.set(tx).expect("Email queue already initialized.");
    
    tokio::spawn(async move {
        while let Some(email) = rx.recv().await {
            tokio::task::spawn_blocking(move || {
                let email_msg  = Message::builder()
                    .from(format!("Key Manager <{}>", email.sender).parse().unwrap())
                    .to(format!("Receiver <{}>", email.recipient).parse().unwrap())
                    .subject(email.subject)
                    .body(email.body)
                    .unwrap();

                let mailer = SmtpTransport::builder_dangerous(env::var("SMTP_SERVER")
                    .expect("SMTP_SERVER must be set").to_string())
                    .port(env::var("SMTP_PORT").expect("SMTP_PORT must be set").to_string().parse().unwrap())
                    .build();
                
                match mailer.send(&email_msg ) {
                    Ok(_) => println!("Email sent successfully!"),
                    Err(e) => eprintln!("Error sending email: {:?}", e),
                }
            });
        }
    });
}
