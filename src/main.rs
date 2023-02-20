use std::time::Duration;

use anyhow::{bail, Context, Result};
use argh::from_env;
use capctl::caps::{Cap, CapState};
use futures_util::TryStreamExt;
use reqwest::ClientBuilder;
use rtnetlink::new_connection;
use tokio::task;
use tokio::time::sleep;

use crate::cli::Args;
use crate::notify::try_notify_systemd;
use crate::periodic::run_periodically;
use crate::update::update_routes;
use crate::url::parse_url;

mod parser;
mod cli;
mod url;
mod notify;
mod update;
mod periodic;

#[tokio::main]
async fn main() -> Result<()> {
    // initialization
    let args: Args = from_env();
    let update_interval = Duration::from_secs((args.interval as u64) * 3600);

    // check if this program have enough privilege
    let init_cap_state = CapState::get_current().context("Failed to get process capabilities")?;
    if !init_cap_state.permitted.has(Cap::NET_ADMIN) {
        eprintln!("routeupd needs CAP_NET_ADMIN to use rtnetlink to modify routing tables!");
        eprintln!("consider running this program as root or setting CAP_NET_ADMIN");
        eprintln!();
        bail!("Not enough privilege")
    }

    // establish netlink connection
    let (conn, handle, _) =
        new_connection()
            .context("Failed to establish a netlink connection")?;
    task::spawn(conn);

    // get the interface id
    let interface =
        handle.link()
            .get()
            .match_name(args.interface.clone())
            .execute()
            .try_next().await
            .with_context(||
                format!("Failed to get the id of the interface with the name {}", args.interface)
            )?
            .with_context(||
                format!("No interface with the name {} was found", args.interface)
            )?;
    let if_id = interface.header.index;

    // setup the reqwest client
    let client =
        ClientBuilder::new()
            .gzip(true)
            .timeout(Duration::from_secs(10))
            .connect_timeout(Duration::from_secs(10))
            .build()
            .context("Failed to build reqwest client")?;

    let v4_url = parse_url(&args.v4_url).context("Failed to parse URL")?;
    let v6_url = parse_url(&args.v6_url).context("Failed to parse URL")?;
    // initialization complete

    update_routes(&client, &handle, &args, if_id, &v4_url, &v6_url).await?;

    if args.daemon {
        try_notify_systemd()?;
        println!("going into sleep for {update_interval:?}");
        sleep(update_interval).await;

        run_periodically(update_interval, || async {
            let r = update_routes(&client, &handle, &args, if_id, &v4_url, &v6_url).await;
            println!("going into sleep for {update_interval:?}");
            r
        }).await?;
    }

    Ok(())
}
