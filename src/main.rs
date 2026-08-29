use google_gmail1::{
    hyper_rustls, //conexiones https se encarga
    hyper_util, //realiza las consultas
    yup_oauth2,
    Gmail,
};

use std::io::{self, Write};

const SCOPES: &[&str] = &[
    "https://www.googleapis.com/auth/gmail.modify",
];

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=================================");
    println!("       Gmail Rust Manager");
    println!("=================================\n");

    // ----------------------------------------
    // 1. Load Google OAuth credentials
    // ----------------------------------------

    let secret = yup_oauth2::read_application_secret("credentials.json")
        .await
        .map_err(|e| format!("Could not read credentials.json: {}", e))?;

    // ----------------------------------------
    // 2. Create HTTPS connector
    // ----------------------------------------

    let connector = hyper_rustls::HttpsConnectorBuilder::new() //permitira crear una conexion https
        .with_native_roots() //cargar los certificados nativos
        .expect("Could not load native certificates") //si no se pueden cargar los certificados, se lanza un error
        .https_only() //solo se permitira https
        .enable_http2() //habilitar http2
        .build();

    // ----------------------------------------
    // 3. Create OAuth authenticator
    // ----------------------------------------

    let executor = hyper_util::rt::TokioExecutor::new();  

    let auth = yup_oauth2::InstalledFlowAuthenticator::with_client( //sistema que manejara la autenticacion 
        secret, //le paso las credenciales que leyo 
        yup_oauth2::InstalledFlowReturnMethod::HTTPRedirect,
        yup_oauth2::client::CustomHyperClientBuilder::from(
            hyper_util::client::legacy::Client::builder(executor)
                .build(connector),
        ),
    )
    .persist_tokens_to_disk("token.json") //guarda todos los tokens aqui
    .build()
    .await?; //singinfica que se debe esperar a que se complete la operacion

    println!("Authentication initialized.");

    // ----------------------------------------
    // 4. Create Gmail HTTP client
    // ----------------------------------------

    let client = hyper_util::client::legacy::Client::builder(
        hyper_util::rt::TokioExecutor::new(),
    )
    .build(
        hyper_rustls::HttpsConnectorBuilder::new()
            .with_native_roots()
            .expect("Could not load native certificates")
            .https_or_http()
            .enable_http2()
            .build(),
    );

    // ----------------------------------------
    // 5. Create Gmail API client
    // ----------------------------------------

    let hub = Gmail::new(client, auth);

    // ----------------------------------------
    // 6. Get Gmail profile
    // ----------------------------------------

    let profile = hub
        .users()
        .get_profile("me") //"me" significa usuario autenticado
        .doit()
        .await?
        .1;

    println!(
        "\nConnected to Gmail account: {}",
        profile.email_address.unwrap_or_default()
    );

    // ----------------------------------------
    // 7. Ask for Gmail search query
    // ----------------------------------------

    println!("\nExamples of Gmail searches:");
    println!("  from:example@gmail.com");
    println!("  subject:newsletter");
    println!("  is:unread");
    println!("  older_than:1y");
    println!("  from:example@gmail.com after:2026/01/01");

    print!("\nEnter Gmail search query: ");
    io::stdout().flush()?;

    let mut query = String::new(); //una variable mutable para guardar el texto
    io::stdin().read_line(&mut query)?; //se espera la respuesta en consola

    let query = query.trim();

    if query.is_empty() {
        println!("Search query cannot be empty.");
        return Ok(());
    }

    // ----------------------------------------
    // 8. Search Gmail
    // ----------------------------------------

    println!("\nSearching Gmail for: {}", query);

    let response = hub
        .users()
        .messages_list("me")
        .q(query) //aqui ya va lo contestado anteriormente del query del cliente
        .max_results(100)
        .doit()
        .await?;

    let messages = response.1.messages.unwrap_or_default();

    if messages.is_empty() {
        println!("\nNo emails found.");
        return Ok(());
    }

    println!("\nFound {} email(s).\n", messages.len());


    // ==========================================
    // 10. Confirmation
    // ==========================================

    print!("Move these emails to Trash? (y/N): ");
    io::stdout().flush()?;

    let mut confirmation = String::new();
    io::stdin().read_line(&mut confirmation)?;

    if confirmation.trim().to_lowercase() != "y" {
        println!("\nNo emails were deleted.");
        return Ok(());
    }

    // ==========================================
    // 11. Move emails to Trash
    // ==========================================

    println!("\nMoving emails to Trash...\n");

    for message in messages {
        if let Some(message_id) = message.id {
            match hub
                .users()
                .messages_trash("me", &message_id) //aqqui ya se pasa a la papelera el correo
                .doit()
                .await
            {
                Ok(_) => {
                    println!("Moved to Trash: {}", message_id);
                }

                Err(error) => {
                    println!(
                        "Could not trash {}: {}",
                        message_id, error
                    );
                }
            }
        }
    }

    println!("\nFinished.");

    Ok(())
}