use clap::Parser;
use gmail_rust::{
    build_hub, fetch_preview, profile_email, search_messages, trash_messages, AppResult, Hub,
};

use std::io::{self, Write};
use std::process::ExitCode;

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

    println!(
        "\nConnected to Gmail account: {}",
        profile_email(&hub).await?
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

    print_preview(&hub, &ids).await;

    if args.dry_run {
        println!("\nDry run: no emails were modified.");
        return Ok(true);
    }

    if !args.yes && !confirm(ids.len())? {
        println!("\nNo emails were deleted.");
        return Ok(true);
    }

    println!("\nMoving emails to Trash...\n");

    let report = trash_messages(&hub, &ids).await;

    for failure in &report.failed {
        println!("Could not trash {}: {}", failure.id, failure.error);
    }

    println!(
        "\nFinished. Moved to Trash: {}. Failed: {}.",
        report.moved,
        report.failed.len()
    );

    Ok(report.failed.is_empty())
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

// ----------------------------------------
// Preview
// ----------------------------------------

async fn print_preview(hub: &Hub, ids: &[String]) {
    for id in ids.iter().take(PREVIEW_LIMIT) {
        match fetch_preview(hub, id).await {
            Ok(email) => println!("  {} | {} | {}", email.date, email.from, email.subject),
            Err(error) => println!("  Could not preview {}: {}", id, error),
        }
    }

    if ids.len() > PREVIEW_LIMIT {
        println!("  ... and {} more.", ids.len() - PREVIEW_LIMIT);
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_yes_only_accepts_explicit_yes() {
        assert!(is_yes("y\n"));
        assert!(is_yes("  YES \r\n"));
        assert!(!is_yes("\n"));
        assert!(!is_yes("n"));
        assert!(!is_yes("yep"));
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
