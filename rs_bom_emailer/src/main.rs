use chrono::Utc;
use lettre::transport::smtp::authentication::Credentials;
use lettre::{Message, SmtpTransport, Transport};
use rand::Rng;
use rs_bom::BOM;

fn main() {
    let bom = BOM::from_default_parser().expect("Failed to get BOM");

    let mut rng = rand::thread_rng();
    let r = rng.gen_range(0, bom.verses().count());
    let random_verse = bom.verses().nth(r).unwrap();
    let now = Utc::now();

    let email = Message::builder()
        .from("Ephraim Kunz <ephraimkunz@gmail.com>".parse().unwrap())
        .to("Ephraim Kunz <ephraimkunz@icloud.com>".parse().unwrap())
        .subject(format!("Random verse for {}", now.format("%A, %B %e")))
        .body(random_verse.to_string())
        .unwrap();

    // Define these at compile time so they'll be inserted into the binary.
    let username = env!("USERNAME").to_string();
    let password = env!("PASSWORD").to_string();
    
    let creds = Credentials::new(username, password);

    // Open a remote connection to gmail
    let mailer = SmtpTransport::relay("smtp.gmail.com")
        .expect("Unabled to connect to gmail transport")
        .credentials(creds)
        .build();

    // Send the email
    match mailer.send(&email) {
        Ok(_) => println!("Email sent successfully!"),
        Err(e) => panic!("Could not send email: {:?}", e),
    }
}
