use std::env;
use systemd::daemon::notify;
use anyhow::Context;

pub fn try_notify_systemd() -> anyhow::Result<()> {
    if env::var("INVOCATION_ID").is_ok() {
        notify(false, [("READY", "1")].iter())
            .context("Failed to notify systemd")?;
    }
    Ok(())
}
