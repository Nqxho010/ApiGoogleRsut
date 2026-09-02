# Gmail Email Manager

A desktop application built with **Rust** that integrates with the **Google Gmail API** to search and manage emails directly from a local environment.

The project was created to provide a simple and controlled way to search for emails using Gmail search queries and move matching messages to the Gmail Trash after user confirmation.

## Overview

Gmail Email Manager uses the **Gmail API** and **OAuth 2.0** authentication to securely connect a Google account with the application.

Once authenticated, the application:

1. Connects to the user's Gmail account
2. Retrieves the authenticated user's Gmail profile
3. Accepts a Gmail search query
4. Retrieves messages matching the query
5. Displays the number of matching emails
6. Requests confirmation before making changes
7. Moves the selected messages to the Gmail Trash

The project focuses on learning and implementing API integration, OAuth authentication, asynchronous programming, and interaction with external cloud services using Rust.

## Application Flow

```text
┌──────────────────────────────┐
│       Start Application      │
└──────────────┬───────────────┘
               ↓
┌──────────────────────────────┐
│     Load Google Credentials  │
└──────────────┬───────────────┘
               ↓
┌──────────────────────────────┐
│       OAuth 2.0 Login        │
└──────────────┬───────────────┘
               ↓
┌──────────────────────────────┐
│      Connect to Gmail API    │
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
      Finish      Confirmation
                      │
                ┌─────┴─────┐
                │           │
               No          Yes
                │           │
                ↓           ↓
             Finish    Move to Trash
                            │
                            ↓
                         Finish
```

## Features

* Google account authentication using OAuth 2.0
* Integration with the Gmail API
* Gmail profile verification
* Support for Gmail search queries
* Retrieval of matching email messages
* User confirmation before modifying emails
* Batch processing of matching messages
* Moving emails to the Gmail Trash
* Local token persistence for subsequent authentications
* Asynchronous API communication

## Gmail Search

The application supports Gmail's standard search syntax.

Examples:

```text
from:example@gmail.com
```

```text
subject:newsletter
```

```text
from:example@gmail.com subject:invoice
```

```text
older_than:1y
```

These queries are sent to Gmail through the API to retrieve matching messages.

## Authentication

The application uses **Google OAuth 2.0** to authenticate the user.

The authentication flow is based on a Google Cloud project and OAuth credentials. During the first execution, the user authorizes the application through their Google account.

After successful authentication, the application stores the token locally so that subsequent executions can reuse the authorization without requiring the complete authentication process again.

### Required Google Configuration

A Google Cloud project must be configured with:

* Gmail API enabled
* OAuth 2.0 credentials
* A configured OAuth consent screen
* The required Gmail API scopes

The application requires a `credentials.json` file containing the Google OAuth client configuration.

> **Security:** Do not commit personal Google credentials, OAuth tokens, or other sensitive authentication files to a public repository.

## Prerequisites

Before running the project, make sure you have:

* [Rust](https://www.rust-lang.org/) installed
* Cargo installed with Rust
* Node.js if required by the project interface/build environment
* A Google account with Gmail
* A Google Cloud project with the Gmail API enabled
* OAuth 2.0 credentials

You can verify the Rust installation with:

```bash
rustc --version
cargo --version
```

## Installation

Clone the repository:

```bash
git clone https://github.com/Nqxho010/ApiGoogleRsut.git
```

Enter the project directory:

```bash
cd ApiGoogleRsut
```

Install and compile the project dependencies:

```bash
cargo build
```

Place the Google OAuth credentials file in the appropriate project location:

```text
credentials.json
```

Then run the application:

```bash
cargo run
```

## Usage

When the application starts, it initializes the Google authentication process.

### 1. Authenticate with Google

The application opens the Google authorization flow. Sign in with the Gmail account you want to manage and grant the required permissions.

### 2. Enter a Gmail Search Query

Enter the Gmail search criteria for the emails you want to find.

For example:

```text
from:example@gmail.com
```

### 3. Review the Results

The application queries the Gmail API and retrieves the messages that match the search criteria.

### 4. Confirm the Action

Before modifying the mailbox, the application asks for confirmation.

If the operation is confirmed, the matching messages are moved to the Gmail Trash.

If the operation is cancelled, no changes are made.

## Technologies

| Technology           | Purpose                                       |
| -------------------- | --------------------------------------------- |
| **Rust**             | Application logic and backend                 |
| **Gmail API**        | Email management and communication with Gmail |
| **Google OAuth 2.0** | User authentication and authorization         |
| **Tokio**            | Asynchronous runtime                          |
| **Hyper**            | HTTP communication                            |
| **Cargo**            | Rust package management and build system      |

## Project Structure

```text
ApiGoogleRsut/
│
├── src/
│   └── ...
│
├── Cargo.toml
├── Cargo.lock
├── credentials.json
├── README.md
└── target/
```

The `src` directory contains the Rust application source code, while `Cargo.toml` defines the project's dependencies and configuration.

## Security Considerations

Because this application interacts directly with a user's Gmail account, authentication credentials must be handled carefully.

Do not upload the following files to a public repository:

```text
credentials.json
token.json
```

If these files are currently tracked by Git, they should be removed from the repository and added to `.gitignore`.

Example:

```gitignore
credentials.json
token.json
.env
target/
```

OAuth permissions should also be limited to the minimum Gmail scopes required by the application.

## Current Limitations

The current version is primarily focused on Gmail API integration, authentication, searching, and moving matching messages to the Trash.

Potential future improvements include:

* Graphical desktop interface
* Email preview before deletion
* Advanced filtering options
* Bulk selection controls
* Undo functionality
* Better error handling and user feedback
* Improved authentication management
* Packaging the application as a standalone desktop executable

## Future Development

The project can be expanded into a complete desktop Gmail management tool with a graphical interface, allowing users to search, review, select, and manage emails without interacting directly with the terminal.

## Author

**José Marín Galán**

Software Development Project

GitHub: [Nqxho010](https://github.com/Nqxho010)

## License

This project currently does not specify a license.

If the project is intended to be distributed or used as open source, an appropriate license can be added in the future.
