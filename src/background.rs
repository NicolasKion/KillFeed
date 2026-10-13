//! What the api runs beside the requests: the jobs a stopped api had
//! taken go back to the queue, the workers start, the scheduler and, when
//! `LISTENER_ENABLED`, the firehose listener. All of them stop on the
//! shutdown signal, a job or a firehose item in hand finished first.

use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;

use crate::server::AppState;

pub async fn start(state: &AppState, cancel: &CancellationToken) -> Vec<JoinHandle<()>> {
    match crate::jobs::release_reserved(&state.pool).await {
        Ok(0) => {}
        Ok(released) => tracing::info!(released, "jobs of a stopped api back in the queue"),
        Err(error) => tracing::error!(%error, "reserved jobs not released"),
    }
    let mut tasks = crate::jobs::spawn_workers(state, cancel);
    if state.config.background.scheduler {
        tasks.extend(crate::scheduler::spawn(state, cancel));
    }
    if state.config.background.listener {
        tasks.push(tokio::spawn(crate::listener::run(
            state.clone(),
            cancel.clone(),
        )));
    } else {
        tracing::info!("the firehose listener is off (LISTENER_ENABLED)");
    }
    tasks
}
