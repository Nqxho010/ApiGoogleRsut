use google_gmail1::{
    api::{BatchModifyMessagesRequest, MessagePartHeader, Scope},
    hyper_rustls, //conexiones https se encarga
    hyper_util, //realiza las consultas
    yup_oauth2,
    Gmail,
};
use serde::Serialize;

pub type Connector =
    hyper_rustls::HttpsConnector<hyper_util::client::legacy::connect::HttpConnector>;
pub type Hub = Gmail<Connector>;
// Send + Sync para poder usar los errores desde otros hilos (lo necesita Tauri)
pub type AppResult<T> = Result<T, Box<dyn std::error::Error + Send + Sync>>;

const SCOPE: Scope = Scope::Modify; //unico permiso que se le pide a Google
const USER: &str = "me"; //"me" significa usuario autenticado
const PAGE_SIZE: u32 = 500; //maximo que permite messages.list por pagina
const BATCH_SIZE: usize = 1000; //maximo de ids que permite batchModify

#[derive(Serialize, Debug, Clone, PartialEq)]
pub struct EmailPreview {
    pub id: String,
    pub date: String,
    pub from: String,
    pub subject: String,
}

#[derive(Serialize, Debug, Clone, PartialEq)]
pub struct TrashFailure {
    pub id: String,
    pub error: String,
}

#[derive(Serialize, Debug, Clone, Default, PartialEq)]
pub struct TrashReport {
    pub moved: usize,
    pub failed: Vec<TrashFailure>,
}

// ----------------------------------------
// Authentication and Gmail client
// ----------------------------------------

// Abre el navegador con la pagina de permiso de Google
struct BrowserDelegate;

impl yup_oauth2::authenticator_delegate::InstalledFlowDelegate for BrowserDelegate {
    fn present_user_url<'a>(
        &'a self,
        url: &'a str,
        _need_code: bool,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<String, String>> + Send + 'a>>
    {
        Box::pin(async move {
            // Si el navegador no abre, el enlace queda en la consola
            println!("Open this link to authorize the app:\n{}", url);
            let _ = open::that(url);
            Ok(String::new())
        })
    }
}

pub async fn build_hub(credentials_path: &str, token_path: &str) -> AppResult<Hub> {
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
    .flow_delegate(Box::new(BrowserDelegate))
    .build()
    .await?;

    let client = hyper_util::client::legacy::Client::builder(hyper_util::rt::TokioExecutor::new())
        .build(connector);

    Ok(Gmail::new(client, auth))
}

// Direccion de correo de la cuenta autenticada
pub async fn profile_email(hub: &Hub) -> AppResult<String> {
    let profile = hub
        .users()
        .get_profile(USER)
        .add_scope(SCOPE)
        .doit()
        .await?
        .1;

    Ok(profile.email_address.unwrap_or_default())
}

// ----------------------------------------
// Search
// ----------------------------------------

// Recorre todas las paginas de resultados, no solo la primera
pub async fn search_messages(hub: &Hub, query: &str) -> AppResult<Vec<String>> {
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

pub async fn fetch_preview(hub: &Hub, id: &str) -> AppResult<EmailPreview> {
    let message = hub
        .users()
        .messages_get(USER, id)
        .format("metadata") //solo cabeceras, no el cuerpo del correo
        .add_metadata_headers("From")
        .add_metadata_headers("Subject")
        .add_metadata_headers("Date")
        .add_scope(SCOPE)
        .doit()
        .await?
        .1;

    let headers = message
        .payload
        .and_then(|payload| payload.headers)
        .unwrap_or_default();

    Ok(EmailPreview {
        id: id.to_string(),
        date: header_value(&headers, "Date").to_string(),
        from: header_value(&headers, "From").to_string(),
        subject: header_value(&headers, "Subject").to_string(),
    })
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
// Move emails to Trash
// ----------------------------------------

fn trash_request(ids: &[String]) -> BatchModifyMessagesRequest {
    BatchModifyMessagesRequest {
        add_label_ids: Some(vec!["TRASH".to_string()]),
        ids: Some(ids.to_vec()),
        remove_label_ids: None,
    }
}

pub async fn trash_messages(hub: &Hub, ids: &[String]) -> TrashReport {
    let mut report = TrashReport::default();

    for batch in ids.chunks(BATCH_SIZE) {
        let result = hub
            .users()
            .messages_batch_modify(trash_request(batch), USER)
            .add_scope(SCOPE)
            .doit()
            .await;

        if result.is_ok() {
            report.moved += batch.len();
            continue;
        }

        // Si el lote falla se intenta correo por correo
        for id in batch {
            match hub
                .users()
                .messages_trash(USER, id) //aqui ya se pasa a la papelera el correo
                .add_scope(SCOPE)
                .doit()
                .await
            {
                Ok(_) => report.moved += 1,

                Err(error) => report.failed.push(TrashFailure {
                    id: id.clone(),
                    error: error.to_string(),
                }),
            }
        }
    }

    report
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
}
