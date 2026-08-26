use google_gmail1::hyper_rustls::HttpsConnectorBuilder;
use google_gmail1::hyper_util::{client::legacy::Client, rt::TokioExecutor};
use google_gmail1::Gmail;
use yup_oauth2::{read_application_secret, InstalledFlowAuthenticator, InstalledFlowReturnMethod};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let secret = read_application_secret("client_secret.json").await?;

    let auth = InstalledFlowAuthenticator::builder(
        secret,
        InstalledFlowReturnMethod::HTTPRedirect,
    )
    .persist_tokens_to_disk("tokencache.json")
    .build()
    .await?;

    // Cliente TLS: Gmail usa https://, no http://
    let https = HttpsConnectorBuilder::new()
        .with_native_roots()?
        .https_or_http()
        .enable_http1()
        .enable_http2()
        .build();

    let client = Client::builder(TokioExecutor::new()).build(https);
    let hub = Gmail::new(client, auth);

    let (_, labels) = hub.users().labels_list("me").doit().await?;
    println!("Etiquetas de Gmail: {labels:#?}");

  

    println!("Conectado a Gmail y solicitando correos!");

    let (_, lista) = hub
    .users()
    .messages_list("me")
    .add_label_ids("INBOX")
    .max_results(10)
    .doit()
    .await?;

    for mensaje in lista.messages.unwrap_or_default() {
        let Some(id) = mensaje.id else { continue };
        let (_, msg) = hub.users().messages_get("me", &id).format("metadata").doit().await?;
        println!("{msg:#?}");
    }

    

    Ok(())
}
