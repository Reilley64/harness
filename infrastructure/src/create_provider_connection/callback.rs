use axum::Router;
use axum::extract::Query;
use axum::http::StatusCode;
use axum::routing::get;
use constant_time_eq::constant_time_eq;
use tokio::net::TcpListener;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;

use super::AuthenticationError;

#[derive(serde::Deserialize)]
pub(super) struct Callback {
    pub state: Option<String>,
    pub code: Option<String>,
    pub client_id: Option<String>,
    pub error: Option<String>,
}

pub(super) struct CallbackServer {
    pub redirect_uri: String,
    pub receiver: mpsc::Receiver<Callback>,
    task: JoinHandle<()>,
}

impl Drop for CallbackServer {
    fn drop(&mut self) {
        self.task.abort();
    }
}

pub(super) async fn start(state: &str) -> Result<CallbackServer, AuthenticationError> {
    let listener = TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, 0)).await?;
    let redirect_uri = format!(
        "http://127.0.0.1:{}/auth/callback",
        listener.local_addr()?.port()
    );
    let (sender, receiver) = mpsc::channel(1);
    let expected_state = state.to_owned();
    let router = Router::new().route(
        "/auth/callback",
        get(move |Query(callback): Query<Callback>| {
            let sender = sender.clone();
            let expected_state = expected_state.clone();
            async move {
                let state_matches = callback.state.as_ref().is_some_and(|state| {
                    constant_time_eq(state.as_bytes(), expected_state.as_bytes())
                });
                if !state_matches {
                    return (
                        StatusCode::BAD_REQUEST,
                        "Invalid OAuth state. Return to Harness.",
                    );
                }
                if sender.try_send(callback).is_err() {
                    return (
                        StatusCode::CONFLICT,
                        "This authorization attempt has already completed.",
                    );
                }
                (
                    StatusCode::OK,
                    "Authorization received. Return to Harness to check the result.",
                )
            }
        }),
    );
    let task = tokio::spawn(async move {
        let _ = axum::serve(listener, router).await;
    });
    Ok(CallbackServer {
        redirect_uri,
        receiver,
        task,
    })
}
