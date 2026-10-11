use clap::Parser;
use google_gmail1::{
    api::{BatchModifyMessagesRequest, MessagePartHeader, Scope},
    hyper_rustls, //conexiones https se encarga
    hyper_util, //realiza las consultas
    yup_oauth2,
    Gmail,
};

use std::io::{self, Write};
use std::process::ExitCode;

type Connector = hyper_rustls::HttpsConnector<hyper_util::client::legacy::connect::HttpConnector>;
type Hub = Gmail<Connector>;
type AppResult<T> = Result<T, Box<dyn std::error::Error>>;

const SCOPE: Scope = Scope::Modify; //unico permiso que se le pide a Google
const USER: &str = "me"; //"me" significa usuario autenticado
const PAGE_SIZE: u32 = 500; //maximo que permite messages.list por pagina
const BATCH_SIZE: usize = 1000; //maximo de ids que permite batchModify
const PREVIEW_LIMIT: usize = 10;

/// Search Gmail and move the matching emails to the Trash.
#[derive(Parser, Debug)]
#[command(version, about)]
struct Args {
    /// Gmail search query (asked interactively if omitted)
    #[arg(short, long)]
    query: Option<String>,

    /// Search and preview only, without modifying any email
    #[arg(long)]
    dry_run: bool,

    /// Skip the confirmation prompt
    #[arg(short, long)]
    yes: bool,

    /// Path to the Google OAuth client file
    #[arg(long, default_value = "credentials.json")]
    credentials: String,

    /// Path where the OAuth token is stored
    #[arg(long, default_value = "token.json")]
    token: String,
}

#[tokio::main]
async fn main() -> ExitCode {
    match run(Args::parse()).await {
        Ok(true) => ExitCode::SUCCESS,
        Ok(false) => ExitCode::FAILURE,
        Err(error) => {
            eprintln!("\nError: {}", error);
            ExitCode::FAILURE
        }
    }
}

// Devuelve false si algun correo no se pudo mover a la papelera
async fn run(args: Args) -> AppResult<bool> {
    println!("=================================");
    println!("       Gmail Rust Manager");
    println!("=================================\n");

    let hub = build_hub(&args.credentials, &args.token).await?;

    println!("Authentication initialized.");

    let profile = hub
        .users()
        .get_profile(USER)
        .add_scope(SCOPE)
        .doit()
        .await?
        .1;

    println!(
        "\nConnected to Gmail account: {}",
        profile.email_address.unwrap_or_default()
    );

    let query = match args.query {
        Some(query) => query,
        None => ask_query()?,
    };
    let query = query.trim();

    if query.is_empty() {
        println!("Search query cannot be empty.");
        return Ok(true);
    }

    println!("\nSearching Gmail for: {}", query);

    let ids = search_messages(&hub, query).await?;

    if ids.is_empty() {
        println!("\nNo emails found.");
        return Ok(true);
    }

    println!("\nFound {} email(s).\n", ids.len());

    preview(&hub, &ids).await;

    if args.dry_run {
        println!("\nDry run: no emails were modified.");
        return Ok(true);
    }

    if !args.yes && !confirm(ids.len())? {
        println!("\nNo emails were deleted.");
        return Ok(true);
    }

    println!("\nMoving emails to Trash...\n");

    let (moved, failed) = trash_messages(&hub, &ids).await;

    println!("\nFinished. Moved to Trash: {}. Failed: {}.", moved, failed);

    Ok(failed == 0)
}

// ----------------------------------------
// Authentication and Gmail client
// ----------------------------------------

async fn build_hub(credentials_path: &str, token_path: &str) -> AppResult<Hub> {
    let secret = yup_oauth2::read_application_secret(credentials_path)
        .await
        .map_err(|e| format!("Could not read {}: {}", credentials_path, e))?;

    let connector = hyper_rustls::HttpsConnectorBuilder::new() //permitira crear una conexion https
        .with_native_roots() //cargar los certificados nativos
        .map_err(|e| format!("Could not load native certificates: {}", e))?
        .https_only() //solo se permitira https
        .enable_http2() //habilitar http2
        .build();

    let auth = yup_oauth2::InstalledFlowAuthenticator::with_client( //sistema que manejara la autenticacion
        secret, //le paso las credenciales que leyo
        yup_oauth2::InstalledFlowReturnMethod::HTTPRedirect,
        yup_oauth2::client::CustomHyperClientBuilder::from(
            hyper_util::client::legacy::Client::builder(hyper_util::rt::TokioExecutor::new())
                .build(connector.clone()),
        ),
    )
    .persist_tokens_to_disk(token_path) //guarda todos los tokens aqui
    .build()
    .await?;

    let client = hyper_util::client::legacy::Client::builder(hyper_util::rt::TokioExecutor::new())
        .build(connector);

    Ok(Gmail::new(client, auth))
}

// ----------------------------------------
// Search
// ----------------------------------------

fn ask_query() -> io::Result<String> {
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

    Ok(query)
}

// Recorre todas las paginas de resultados, no solo la primera
async fn search_messages(hub: &Hub, query: &str) -> AppResult<Vec<String>> {
    let mut ids = Vec::new();
    let mut page_token: Option<String> = None;

    loop {
        let mut call = hub
            .users()
            .messages_list(USER)
            .q(query)
            .max_results(PAGE_SIZE)
            .add_scope(SCOPE);

        if let Some(token) = &page_token {
            call = call.page_token(token);
        }

        let response = call.doit().await?.1;

        ids.extend(
            response
                .messages
                .unwrap_or_default()
                .into_iter()
                .filter_map(|message| message.id),
        );

        page_token = response.next_page_token;

        if page_token.is_none() {
            return Ok(ids);
        }
    }
}

// ----------------------------------------
// Preview
// ----------------------------------------

async fn preview(hub: &Hub, ids: &[String]) {
    for id in ids.iter().take(PREVIEW_LIMIT) {
        let result = hub
            .users()
            .messages_get(USER, id)
            .format("metadata") //solo cabeceras, no el cuerpo del correo
            .add_metadata_headers("From")
            .add_metadata_headers("Subject")
            .add_metadata_headers("Date")
            .add_scope(SCOPE)
            .doit()
            .await;

        match result {
            Ok((_, message)) => {
                let headers = message
                    .payload
                    .and_then(|payload| payload.headers)
                    .unwrap_or_default();

                println!(
                    "  {} | {} | {}",
                    header_value(&headers, "Date"),
                    header_value(&headers, "From"),
                    header_value(&headers, "Subject")
                );
            }

            Err(error) => {
                println!("  Could not preview {}: {}", id, error);
            }
        }
    }

    if ids.len() > PREVIEW_LIMIT {
        println!("  ... and {} more.", ids.len() - PREVIEW_LIMIT);
    }
}

fn header_value<'a>(headers: &'a [MessagePartHeader], name: &str) -> &'a str {
    headers
        .iter()
        .find(|header| {
            header
                .name
                .as_deref()
                .is_some_and(|n| n.eq_ignore_ascii_case(name))
        })
        .and_then(|header| header.value.as_deref())
        .unwrap_or("(none)")
}

// ----------------------------------------
// Confirmation
// ----------------------------------------

fn confirm(count: usize) -> io::Result<bool> {
    print!("\nMove these {} email(s) to Trash? (y/N): ", count);
    io::stdout().flush()?;

    let mut confirmation = String::new();
    io::stdin().read_line(&mut confirmation)?;

    Ok(is_yes(&confirmation))
}

fn is_yes(input: &str) -> bool {
    matches!(input.trim().to_lowercase().as_str(), "y" | "yes")
}

// ----------------------------------------
// Move emails to Trash
// ----------------------------------------

fn trash_request(ids: &[String]) -> BatchModifyMessagesRequest {
    BatchModifyMessagesRequest {
        add_label_ids: Some(vec!["TRASH".to_string()]),
        ids: Some(ids.to_vec()),
        remove_label_ids: None,
    }
}

// Devuelve (movidos, fallidos)
async fn trash_messages(hub: &Hub, ids: &[String]) -> (usize, usize) {
    let mut moved = 0;
    let mut failed = 0;

    for batch in ids.chunks(BATCH_SIZE) {
        let result = hub
            .users()
            .messages_batch_modify(trash_request(batch), USER)
            .add_scope(SCOPE)
            .doit()
            .await;

        match result {
            Ok(_) => {
                moved += batch.len();
                println!("Moved to Trash: {} email(s)", batch.len());
            }

            // Si el lote falla se intenta correo por correo
            Err(error) => {
                println!("Batch failed ({}), retrying one by one...", error);

                for id in batch {
                    match hub
                        .users()
                        .messages_trash(USER, id) //aqui ya se pasa a la papelera el correo
                        .add_scope(SCOPE)
                        .doit()
                        .await
                    {
                        Ok(_) => moved += 1,

                        Err(error) => {
                            failed += 1;
                            println!("Could not trash {}: {}", id, error);
                        }
                    }
                }
            }
        }
    }

    (moved, failed)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn header(name: &str, value: &str) -> MessagePartHeader {
        MessagePartHeader {
            name: Some(name.to_string()),
            value: Some(value.to_string()),
        }
    }

    #[test]
    fn is_yes_only_accepts_explicit_yes() {
        assert!(is_yes("y\n"));
        assert!(is_yes("  YES \r\n"));
        assert!(!is_yes("\n"));
        assert!(!is_yes("n"));
        assert!(!is_yes("yep"));
    }

    #[test]
    fn header_value_ignores_case_and_handles_missing() {
        let headers = vec![header("from", "a@example.com"), header("Subject", "Hi")];

        assert_eq!(header_value(&headers, "From"), "a@example.com");
        assert_eq!(header_value(&headers, "Subject"), "Hi");
        assert_eq!(header_value(&headers, "Date"), "(none)");
    }

    #[test]
    fn trash_request_adds_trash_label_to_all_ids() {
        let ids = vec!["1".to_string(), "2".to_string()];
        let request = trash_request(&ids);

        assert_eq!(request.add_label_ids, Some(vec!["TRASH".to_string()]));
        assert_eq!(request.ids, Some(ids));
        assert_eq!(request.remove_label_ids, None);
    }

    #[test]
    fn ids_are_split_into_batches_of_at_most_batch_size() {
        let ids: Vec<String> = (0..2500).map(|n| n.to_string()).collect();
        let sizes: Vec<usize> = ids.chunks(BATCH_SIZE).map(|batch| batch.len()).collect();

        assert_eq!(sizes, vec![1000, 1000, 500]);
    }

    #[test]
    fn args_default_to_interactive_mode() {
        let args = Args::parse_from(["gmail_rust"]);

        assert_eq!(args.query, None);
        assert!(!args.dry_run);
        assert!(!args.yes);
        assert_eq!(args.credentials, "credentials.json");
        assert_eq!(args.token, "token.json");
    }

    #[test]
    fn args_parse_flags() {
        let args = Args::parse_from(["gmail_rust", "-q", "is:unread", "--dry-run", "--yes"]);

        assert_eq!(args.query.as_deref(), Some("is:unread"));
        assert!(args.dry_run);
        assert!(args.yes);
    }
}
