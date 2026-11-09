use crate::ports::PortFuture;
use parking_lot::Mutex;
use std::collections::HashMap;
use std::sync::Arc;
use tokio_util::sync::CancellationToken;

type Error = Box<dyn std::error::Error + Send + Sync>;

#[derive(Debug)]
pub enum StartError {
    Closing,
    AlreadyOwned,
}

impl std::fmt::Display for StartError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Closing => "Execution runtime is closing",
            Self::AlreadyOwned => "Execution runtime already owns this child",
        })
    }
}

impl std::error::Error for StartError {}

#[derive(Clone, Default)]
pub struct ChildRuntime {
    state: Arc<Mutex<State>>,
    shutdown: Arc<tokio::sync::Mutex<Option<tokio::task::JoinHandle<Vec<Error>>>>>,
}

#[derive(Default)]
struct State {
    closing: bool,
    tasks: HashMap<String, Execution>,
    retired: Vec<Execution>,
}

struct Execution {
    parent_id: String,
    execution_id: String,
    cancellation: CancellationToken,
    task: tokio::task::JoinHandle<Result<(), Error>>,
}

impl ChildRuntime {
    pub fn start(
        &self,
        parent_id: String,
        execution_id: String,
        operation: PortFuture<Result<(), Error>>,
        cancellation: CancellationToken,
    ) -> Result<(), StartError> {
        self.start_owned(
            parent_id,
            execution_id.clone(),
            execution_id,
            operation,
            false,
            cancellation,
        )
    }

    pub fn start_run(
        &self,
        parent_id: String,
        execution_id: String,
        run_key: String,
        operation: PortFuture<Result<(), Error>>,
        cancellation: CancellationToken,
    ) -> Result<(), StartError> {
        self.start_owned(
            parent_id,
            execution_id,
            run_key,
            operation,
            true,
            cancellation,
        )
    }

    fn start_owned(
        &self,
        parent_id: String,
        execution_id: String,
        run_key: String,
        operation: PortFuture<Result<(), Error>>,
        retry_finished: bool,
        cancellation: CancellationToken,
    ) -> Result<(), StartError> {
        let mut state = self.state.lock();
        if state.closing {
            return Err(StartError::Closing);
        }
        if state
            .tasks
            .get(&run_key)
            .is_some_and(|task| !retry_finished || !task.task.is_finished())
            || state.retired.len() >= 4096
            || state
                .tasks
                .values()
                .any(|task| task.execution_id == execution_id && !task.task.is_finished())
        {
            return Err(StartError::AlreadyOwned);
        }
        if let Some(prior) = state.tasks.remove(&run_key) {
            state.retired.push(prior);
        }
        state.tasks.insert(
            run_key,
            Execution {
                parent_id,
                execution_id,
                cancellation,
                task: tokio::spawn(operation),
            },
        );
        Ok(())
    }

    pub fn cancel(&self, execution_id: &str) {
        for execution in self
            .state
            .lock()
            .tasks
            .values()
            .filter(|execution| execution.execution_id == execution_id)
        {
            execution.cancellation.cancel();
        }
    }

    pub fn active(&self) -> Vec<(String, String)> {
        self.state
            .lock()
            .tasks
            .iter()
            .filter(|(_, execution)| !execution.task.is_finished())
            .map(|(_, execution)| (execution.parent_id.clone(), execution.execution_id.clone()))
            .collect()
    }

    pub async fn drain(&self) -> Vec<Error> {
        let mut shutdown = self.shutdown.lock().await;
        if shutdown.is_none() {
            let tasks = {
                let mut state = self.state.lock();
                state.closing = true;
                for execution in state.tasks.values() {
                    execution.cancellation.cancel();
                }
                let mut tasks = std::mem::take(&mut state.retired);
                tasks.extend(std::mem::take(&mut state.tasks).into_values());
                tasks
            };
            *shutdown = Some(tokio::spawn(async move {
                let mut errors = Vec::new();
                for execution in tasks {
                    match execution.task.await {
                        Ok(Ok(())) => {}
                        Ok(Err(error)) => errors.push(error),
                        Err(error) => errors.push(Box::new(error) as Error),
                    }
                }
                errors
            }));
        }
        let result = match shutdown.as_mut() {
            Some(task) => match task.await {
                Ok(errors) => errors,
                Err(error) => vec![Box::new(error) as Error],
            },
            None => Vec::new(),
        };
        *shutdown = None;
        result
    }
}
