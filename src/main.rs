use tokio_util::sync::CancellationToken;

#[tokio::main]
async fn main() {
    // Structured logs; tune with RUST_LOG (e.g. RUST_LOG=killfeed=debug).
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    // Local configuration from .env, if present; real environment wins.
    dotenvy::dotenv().ok();

    let config = killfeed::config::Config::from_env();
    let addr = killfeed::server::bind_addr();

    let pool = killfeed::db::connect().await.expect("database connection");
    killfeed::db::migrate(&pool)
        .await
        .expect("database migrations");

    let state = killfeed::server::AppState::new(pool, config, killfeed::clock::Clock::system());
    let shutdown = CancellationToken::new();
    let background = killfeed::background::start(&state, &shutdown).await;
    let app = killfeed::server::router(state);

    tracing::info!("listening on http://{addr}");
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .expect("bind address");
    let signal = shutdown.clone();
    axum::serve(listener, app)
        .with_graceful_shutdown(async move {
            shutdown_signal().await;
            tracing::info!("shutting down");
            signal.cancel();
        })
        .await
        .expect("serve");

    // The listener stores its sequence and the workers finish their job.
    for task in background {
        let _ = task.await;
    }
}

/// SIGTERM (docker stop) or Ctrl-C.
async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c().await.ok();
    };
    #[cfg(unix)]
    let terminate = async {
        if let Ok(mut signal) =
            tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
        {
            signal.recv().await;
        }
    };
    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();
    tokio::select! {
        () = ctrl_c => {}
        () = terminate => {}
    }
}
