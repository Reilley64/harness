pub struct SteeringRequest {
    pub id: String,
    pub text: String,
    pub accepted: tokio::sync::oneshot::Sender<bool>,
}
