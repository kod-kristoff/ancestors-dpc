mod event;
mod event_log;
mod persistence;
mod services;

fn main() -> eyre::Result<()> {
    let (event_writer, event_reader) = event_log::new_in_memory_shared()?;

    let svc_ctr = services::ServiceControl::new();
    ctrlc::set_handler({
        let svc_ctr = svc_ctr.clone();
        move || {
            eprintln!("Stopping all services ...");
            svc_ctr.send_stop_to_all();
        }
    })?;

    for handle in vec![svc_ctr.spawn_loop(services::Tui::new(event_writer.clone()))] {
        handle.join()?;
    }
    Ok(())
}
