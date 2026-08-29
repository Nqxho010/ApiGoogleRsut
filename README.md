# Flowchart: Gmail API Email Trash Script with Rust

This diagram illustrates the logical lifecycle of the Rust application to authenticate via OAuth 2.0, query Gmail messages based on a user string, and safely batch-move those emails to the system trash.

## Execution Flowchart

```text
 ┌─────────────────────────────────────────────────────────┐
 │               [ PHASE 1: OAUTH 2.0 AUTHENTICATION ]     │
 └───────────────────────────┬─────────────────────────────┘
                             ▼
                 📂 Read 'credentials.json'
                             │
                             ▼
                 🌐 Build HTTPS Client
                             │
                             ▼
                 🔐 Initiate OAuth 2.0 Flow
                             │
                             ▼
                 🌐 Google Login via Browser
                             │
                             ▼
                 💾 Save Local Token ('token.json')
                             │
                             ▼
                 🛠️  Instantiate Gmail API Client
                             │
                             ▼
 ┌─────────────────────────────────────────────────────────┐
 │               [ PHASE 2: INITIALIZATION & SEARCH ]       │
 └───────────────────────────┬─────────────────────────────┘
                             ▼
                 🔌 Connect to Gmail Cloud
                             │
                             ▼
                 👤 Fetch User Profile ("me")
                             │
                             ▼
                 📧 Print Active Account to Terminal
                             │
                             ▼
                 ⌨️  Prompt User: Input Search Query
                             │
                             ▼
                 🔍 Request: messages_list()
                             │
                             ▼
                    ❓ Any results found?
                           /     \
                         NO       YES
                        /           \
                       ▼             ▼
                 ┌───────────┐   🆔 Extract Message IDs
                 │  END (0)  │       │
                 └───────────┘       ▼
 ┌───────────────────────────────────┴─────────────────────┐
 │               [ PHASE 3: CONFIRMATION & DELETION ]      │
 └───────────────────────────┬─────────────────────────────┘
                             ▼
                 ⚠️  Prompt Action Confirmation [y/N]
                             │
                             ▼
                     ❓ Did User Confirm?
                           /     \
                         NO       YES
                        /           \
                       ▼             ▼
                 ┌───────────┐   🗑️  Request: messages_trash()
                 │  END (0)  │       │
                 └───────────┘       ▼
                                 🚀 Move Emails to Trash
                                     │
                                     ▼
                               ┌───────────┐
                               │  END (0)  │
                               └───────────┘
```

## Component Breakdown

### Phase 1: Security & Identity
* **Credentials:** Loads the local OAuth App credentials configuration.
* **OAuth 2.0:** Spawns a local loopback server or browser verification context to retrieve a session token.
* **Token Storage:** Encrypts or caches the token state to allow subsequent runs to skip authorization.

### Phase 2: Interacting with Services
* **Validation:** Tests API capabilities by pulling metadata for the `"me"` email handle.
* **Filtering:** Uses standard Gmail query strings (e.g., `from:example@mail.com subject:test`) via `messages_list()`.

### Phase 3: Mutating Cloud State
* **Safe Halts:** Forces interactive consent before modifying remote mailboxes.
* **Execution:** Iterates over the matching message collections to execute the destructive cloud operations securely.
