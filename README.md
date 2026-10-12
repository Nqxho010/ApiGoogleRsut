# Gmail Manager

A **Rust** application that connects to the **Gmail API** to search emails with Gmail's own query syntax and move the ones you choose to the Trash.

It comes in two forms that share the same code:

* **Desktop app** (Tauri): search, review the results in a table, tick the emails you want and move them to the Trash.
* **Command-line tool**: search, preview and move every matching email to the Trash, with a dry-run mode.

The project was built to learn API integration, OAuth 2.0, asynchronous Rust and desktop packaging with Tauri.

## Features

* Google account authentication with OAuth 2.0, opening the browser automatically
* Local token storage, so authorization is only needed once
* Full Gmail search syntax (`from:`, `older_than:`, `category:`, `has:attachment`, ...)
* Retrieval of every matching email, following all result pages
* Preview of sender, subject and date before anything is modified
* Confirmation before moving emails to the Trash
* Batch processing (up to 1000 emails per request), with a one-by-one fallback
* Nothing is deleted permanently: emails go to the Gmail Trash, where they stay for 30 days

Desktop app only:

* Per-email selection with checkboxes, and a select-all for the loaded rows
* Search shortcuts for common filters
* Automatic connection on start when a token already exists
* Light and dark theme following the system setting

Command-line tool only:

* `--dry-run` to see what a query matches without changing anything
* `--yes` to skip the confirmation, for scripts
* Non-zero exit status when any email could not be moved

## How It Works

```text
┌──────────────────────────────┐
│     Load Google Credentials  │
└──────────────┬───────────────┘
               ↓
┌──────────────────────────────┐
│       OAuth 2.0 Login        │
└──────────────┬───────────────┘
               ↓
┌──────────────────────────────┐
│      Enter Search Query      │
└──────────────┬───────────────┘
               ↓
┌──────────────────────────────┐
│      Search Gmail Messages   │
└──────────────┬───────────────┘
               ↓
        ┌──────┴──────┐
        │             │
      No results    Results
        │             │
        ↓             ↓
      Finish    Preview / Select
                      │
                      ↓
                 Confirmation
                      │
                ┌─────┴─────┐
                │           │
               No          Yes
                │           │
                ↓           ↓
             Finish    Move to Trash
```

## Project Structure

```text
ApiGoogleRsut/
│
├── src/
│   ├── lib.rs              Gmail logic shared by both front ends
│   └── main.rs             Command-line tool
│
├── ui/                     Desktop app
│   ├── src/
│   │   ├── index.html
│   │   ├── main.js
│   │   └── styles.css
│   └── src-tauri/
│       ├── src/lib.rs      Tauri commands that call the shared library
│       ├── icons/
│       └── tauri.conf.json
│
├── .github/workflows/rust.yml
├── Cargo.toml
└── README.md
```

`src/lib.rs` holds everything that talks to Gmail (`build_hub`, `search_messages`, `fetch_preview`, `trash_messages`). The command-line tool and the desktop app are thin layers on top of it.

## Google Configuration

Both front ends need a Google Cloud project with:

* The Gmail API enabled
* An OAuth consent screen, with your Google account added as a test user
* An OAuth 2.0 client of type **Desktop app**

Download the client file from the Google Cloud Console and name it `credentials.json`. Where it goes depends on which front end you use (see below).

## Prerequisites

* [Rust](https://www.rust-lang.org/) with Cargo
* A Google account with Gmail and the Google configuration above

For the desktop app, also:

* The Tauri CLI:

  ```bash
  cargo install tauri-cli --version "^2"
  ```

* The [Tauri system dependencies](https://tauri.app/start/prerequisites/) for your platform. On Windows these are the Microsoft C++ Build Tools and WebView2, which Windows 11 already includes.

The desktop app has been built and tested on Windows only.

## Getting the Code

```bash
git clone https://github.com/Nqxho010/ApiGoogleRsut.git
```

```bash
cd ApiGoogleRsut
```

## Desktop App

### Credentials

The app reads `credentials.json` from its data folder and stores `token.json` next to it. On Windows:

```text
%APPDATA%\com.nqxho010.gmailmanager\
```

Create that folder and copy your `credentials.json` into it.

### Run in development

```bash
cd ui
```

```bash
cargo tauri dev
```

### Build the installer

```bash
cd ui
```

```bash
cargo tauri build
```

The results are written to `ui/src-tauri/target/release/`:

| File                                            | Description                  |
| ----------------------------------------------- | ---------------------------- |
| `ui.exe`                                        | Standalone executable        |
| `bundle/nsis/Gmail Manager_0.1.0_x64-setup.exe` | Installer                    |
| `bundle/msi/Gmail Manager_0.1.0_x64_en-US.msi`  | Installer in MSI format      |

The installer is not code-signed, so Windows shows a SmartScreen warning the first time it runs.

To change the app icon, replace the source image and regenerate every size:

```bash
cargo tauri icon path/to/logo.png
```

If the old icon still appears in the built executable, run `cargo clean --release -p ui` inside `ui/src-tauri` and build again.

### Using the app

1. On first start, click **Conectar con Gmail**. The browser opens on the Google authorization page; sign in and grant access. Later starts connect automatically.
2. Type a Gmail search, or click one of the shortcuts, and press **Buscar**.
3. The first 25 matching emails are listed. Use **Cargar más** to load the next ones.
4. Tick the emails to remove. The checkbox in the header selects all loaded rows.
5. Press **Mover a la papelera** and confirm. Only the ticked emails are moved.

## Command-Line Tool

Place `credentials.json` in the project root, then:

```bash
cargo run
```

Without arguments the tool asks for the search query, shows the first 10 matches and asks for confirmation before moving **all** matching emails to the Trash.

| Option                  | Description                                             |
| ----------------------- | ------------------------------------------------------- |
| `-q`, `--query <QUERY>` | Gmail search query (asked interactively if omitted)     |
| `--dry-run`             | Search and preview only, without modifying any email    |
| `-y`, `--yes`           | Skip the confirmation prompt                            |
| `--credentials <PATH>`  | OAuth client file (default: `credentials.json`)         |
| `--token <PATH>`        | Where the OAuth token is stored (default: `token.json`) |

To check what a query would match without changing anything:

```bash
cargo run -- --query "older_than:1y" --dry-run
```

## Gmail Search Examples

```text
from:example@gmail.com
```

```text
from:example@gmail.com subject:invoice
```

```text
category:promotions older_than:1y
```

```text
is:unread has:attachment
```

A date-only query such as `older_than:1y` usually matches far more than expected, including receipts and personal emails. Narrow it by sender or category, and check the results before confirming.

## Development

Checks for the shared library and the command-line tool, which are also what the CI workflow runs:

```bash
cargo clippy --all-targets -- -D warnings
```

```bash
cargo test
```

The CI workflow does not build the desktop app. To check it locally:

```bash
cd ui/src-tauri
```

```bash
cargo check
```

## Technologies

| Technology           | Purpose                                       |
| -------------------- | --------------------------------------------- |
| **Rust**             | Application logic                             |
| **Gmail API**        | Email search and management                   |
| **Google OAuth 2.0** | User authentication and authorization         |
| **Tokio**            | Asynchronous runtime                          |
| **Hyper**            | HTTP communication                            |
| **Tauri**            | Desktop window and packaging                  |
| **HTML, CSS, JS**    | Desktop interface                             |
| **Clap**             | Command-line argument parsing                 |

## Security Considerations

The application acts on a real Gmail account, so its credential files must be handled carefully.

* `credentials.json` and `token.json` are listed in `.gitignore` and must never be committed. `token.json` grants access to the mailbox.
* If a credentials file was ever committed, removing it is not enough because it stays in the Git history: reset the client secret in the Google Cloud Console.
* The application only requests the `gmail.modify` scope, which allows moving messages to the Trash but not deleting them permanently.

## Current Limitations

* Each installation needs its own `credentials.json`, and the Google account must be a test user of the Google Cloud project. The app cannot be handed to other people as is.
* The desktop app has only been built and tested on Windows.
* Email previews are fetched one at a time, so loading each page of 25 takes a few seconds.
* There is no undo inside the app; emails are restored from the Gmail Trash.
* The desktop interface is in Spanish only.

## Author

**José Marín Galán**

Software Development Project

GitHub: [Nqxho010](https://github.com/Nqxho010)

## License

This project currently does not specify a license.

If the project is intended to be distributed or used as open source, an appropriate license can be added in the future.
