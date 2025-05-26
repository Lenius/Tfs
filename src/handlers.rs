use crate::{realtime::WsCommand, tfs::get_tfs_work_item, tool::generate_playwright_spec, watcher_loop, SharedState};


pub fn handle_template(id: u32) -> String {
    let f = get_tfs_work_item(id);
    let result: String = match f {
        Ok(json) => {
            _ = generate_playwright_spec(format!("{}.spec.ts",&id.to_string()), &json.title, &json.steps);
            serde_json::to_string_pretty(&json).unwrap()
        }
        Err(err) => {
            let err_msg = format!("Kunne ikke hente work item: {}", err);
            err_msg
        }
    };

    format!("Template done, TFS svar: {}", result)
}

pub fn handle_start(state: SharedState) -> String {
    let mut st = state.lock().unwrap();
    if st.watcher.is_none() {
        let (tx, rx) = crossbeam_channel::bounded::<()>(1);
        let handle = std::thread::spawn(move || watcher_loop(rx));
        st.stop_tx = Some(tx);
        st.watcher = Some(handle);
        tracing::info!("Watcher startet via websocket");
        "Watcher startet".to_string()
    } else {
        "Watcher kører allerede".to_string()
    }
}

pub fn handle_stop(state: SharedState) -> String {
    let mut st = state.lock().unwrap();
    if let Some(tx) = st.stop_tx.take() {
        let _ = tx.send(());
        if let Some(handle) = st.watcher.take() {
            let _ = handle.join();
        }
        tracing::info!("Watcher stoppet via websocket");
        "Watcher stoppet".to_string()
    } else {
        "Watcher kører ikke".to_string()
    }
}

pub fn handle_kill() -> String {
    tracing::info!("Kill kaldt, lukker app via websocket");
    // Lukker serveren efter kort delay
    tokio::spawn(async {
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        std::process::exit(0);
    });
    "App lukker ned...".to_string()
}

pub fn handle_ws_command(cmd: WsCommand, state: SharedState) -> String {
    match cmd {
        WsCommand::Start => handle_start(state),
        WsCommand::Stop => handle_stop(state),
        WsCommand::Kill => handle_kill(),
        WsCommand::Template { id } => handle_template(id)
    }
}