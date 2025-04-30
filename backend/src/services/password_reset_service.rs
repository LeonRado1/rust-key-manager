use lettre::{Message, SmtpTransport, Transport};
use lettre::transport::smtp::authentication::Credentials;
use std::env;
use tokio::sync::mpsc::{Sender, Receiver};
use once_cell::sync::OnceCell;

pub struct EmailRequest {
    pub sender: String,
    pub recipient: String,
    pub reset_password: String // Message with the reset password
}

/// Create global queue for email requests, initialized only once
///
/// From lib:
/// A thread-safe cell which can be written to only once.
/// OnceCell provides & references to the contents without RAII guards.
static GLOBAL_EMAIL_QUEUE: OnceCell<Sender<EmailRequest>> = OnceCell::new();

/// To add a new request to send an email in the queue
pub async fn enqueue_email_reset_password(sender: String, recipient: String, reset_password: String) {
    // Create a new mail request
    let mail_request = EmailRequest {
        sender,
        recipient,
        reset_password
    };

    // Check existence of the email queue(transmitter)
    // Use get to get the value of the cell
    if let Some(tx) = GLOBAL_EMAIL_QUEUE.get() {
        // Try to send the request to the queue, transmitter send the request
        if tx.send(mail_request).await.is_err() {
            eprintln!("Fail to send email request");
        }
        return;
    }

    eprintln!("Not initialized email service, transmitter does not exist");
}

/// https://github.com/Wulf/create-rust-app/blob/main/create-rust-app/src/mailer.rs
/// A mailer library for Rust
/// https://github.com/lettre/lettre
/// Was created with using Lettre examples
/// `Lettre` is a `synchronous` email library for sending emails in Rust.
/// Use tokio::task::spawn_blocking to run the email sending in a separate thread.
pub fn init_email_reset_password_service() {
    // mpsc - A multi-producer, single-consumer queue for sending values between asynchronous tasks.
    // Where `tx` - transmitter, `rx` - receiver
    // Send and receive EmailRequest structs
    // Create a channel with a size of 50 EmailRequest objects
    // Check docs: Creates a bounded mpsc channel for communicating between asynchronous tasks with backpressure...
    // Was created with using UI helps to create a new channel to have a queue for email requests.
    // Max channel size is 20, to avoid blocking the sender
    let (tx, mut rx): (Sender<EmailRequest>, Receiver<EmailRequest>) = tokio::sync::mpsc::channel(20);
    // Initializes the queue with the transmitter
    // Sets the contents of this cell to value.
    // Returns Ok(()) if the cell was empty and Err(value) if it was full.
    GLOBAL_EMAIL_QUEUE.set(tx).expect("Email queue already initialized");

    // `tokio::spawn` to spawn a new asynchronous task for
    // Using `move` to transfer ownership of the variables into the async block
    tokio::spawn(async move {
        // From lib:
        // Spawns a new asynchronous task, returning a JoinHandle for it.
        // The provided future will start running in the background immediately when spawn is called, even if you don't await the returned JoinHandle.
        // Check the `email` request queue for new requests, and process them
        while let Some(email) = rx.recv().await {
            tokio::task::spawn_blocking(move || {
                // Create a new later, and fill it with the email request data from the queue
                let email_msg  = Message::builder()
                    .from(format!("Key Manager <{}>", email.sender).parse().unwrap())
                    .to(format!("Receiver <{}>", email.recipient).parse().unwrap())
                    .subject("Reset Password")
                    .body(format!("Your secret reset token is: \n{}\nNow you can reset your password using this token.", email.reset_password))
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
                    Ok(_) => println!("Email sent successfully!"),
                    Err(e) => eprintln!("Error sending email: {:?}", e),
                }
            });
        }
    });
}
