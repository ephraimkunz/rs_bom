use chrono::Local;
use lettre::message::header;
use lettre::transport::smtp::authentication::Credentials;
use lettre::{Message, SmtpTransport, Transport};
use rand::Rng;
use rs_bom::BOM;

fn main() {
    let bom = BOM::from_default_parser().expect("Failed to get BOM");

    let mut rng = rand::thread_rng();
    let r = rng.gen_range(0..bom.verses().count());
    let random_verse = bom.verses().nth(r).unwrap();
    let now = Local::now();

    let email = Message::builder()
        .from("Ephraim Kunz <ephraimkunz@gmail.com>".parse().unwrap())
        .to("Ephraim Kunz <ephraimkunz@icloud.com>".parse().unwrap())
        .subject(format!("Random verse for {}", now.format("%A, %B %e")))
        .header(header::ContentType::TEXT_HTML)
        .body(random_verse.to_html_string())
        .unwrap();

    // Define these at compile time so they'll be inserted into the binary.
    match (option_env!("USERNAME"), option_env!("PASSWORD")) {
        (Some(username), Some(password)) => {
            let creds = Credentials::new(username.to_string(), password.to_string());

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
        _ => panic!("Could not read USERNAME or PASSWORD environment variables"),
    }
}
