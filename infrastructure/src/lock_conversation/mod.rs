use std::path::Path;

use application::ports::ConversationGuard;
use application::ports::PortFuture;

use crate::conversations;

pub fn lock_conversation(
    directory: &Path,
) -> impl Fn(String) -> PortFuture<Result<ConversationGuard, conversations::StorageError>>
+ Send
+ Sync
+ use<> {
    let directory = directory.to_owned();
    move |id| {
        let directory = directory.clone();
        Box::pin(async move {
            Ok(ConversationGuard(Box::new(
                conversations::lock(&directory, &id).await?,
            )))
        })
    }
}
