use std::env;
use std::future::ready;
use std::time::Duration;

use anyhow::{bail, Context, Result};
use argh::from_env;
use capctl::caps::{Cap, CapState};
use futures_util::{StreamExt, TryStreamExt};
use futures_util::stream::iter;
use ipnet::{Ipv4Net, Ipv6Net};
use reqwest::{Client, ClientBuilder, Url};
use rtnetlink::{Handle, new_connection};
use rtnetlink::IpVersion::{V4, V6};
use systemd::daemon::notify;
use tokio::time::sleep;

use crate::cli::Args;
use crate::parser::read_lines;
use crate::url::parse_url;

mod parser;
mod cli;
mod url;

#[tokio::main]
async fn main() -> Result<()> {
    // parse arguments
    let args: Args = from_env();

    // parse interval string, if supplied
    let update_interval = Duration::from_secs((&args.interval * 3600) as u64);

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
    tokio::spawn(conn);

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

    update_routes(
        &client, &handle, &args,
        if_id, &v4_url, &v6_url,
    ).await?;
    try_notify_systemd()?;

    if args.daemon {
        loop {
            sleep(update_interval).await;
            update_routes(
                &client, &handle, &args,
                if_id, &v4_url, &v6_url,
            ).await?
        }
    }

    Ok(())
}

async fn update_routes(
    client: &Client,
    handle: &Handle,
    arg: &Args,
    if_id: u32,
    v4_url: &Url,
    v6_url: &Url,
) -> Result<()> {

    // fetch IPv4 networks
    let v4_req =
        client
            .get(v4_url.clone())
            .build()
            .context("Failed to build the request to fetch IPv4 CIDRs")?;
    let v4_resp =
        client
            .execute(v4_req).await
            .and_then(|r| r.error_for_status())
            .context("Request to fetch IPv4 CIDRs failed")?
            .text().await
            .context("Failed to get the IPv4 response body")?;
    let v4_nets = read_lines::<Ipv4Net>(&v4_resp);

    // get existing routes in the table
    let routes =
        handle.route()
            .get(V4)
            .execute()
            .try_filter(|x| ready(x.header.table == arg.table));

    // delete existing routes
    routes.try_for_each_concurrent(
        10,
        |r| handle.route().del(r).execute(),
    ).await.context("Failed to delete route")?;

    let mut v4_count = 0u64;

    // add the routes
    iter(v4_nets)
        .map(Ok)
        .try_for_each_concurrent(
            10,
            |n| {
                v4_count += 1;
                handle.route()
                    .add()
                    .v4()
                    .table(arg.table)
                    .output_interface(if_id)
                    .destination_prefix(n.addr(), n.prefix_len())
                    .execute()
            },
        ).await
        .context("Failed to add route")?;

    if !arg.no_v6 {
        // fetch IPv6 networks
        let v6_req =
            client
                .get(v6_url.clone())
                .build()
                .context("Failed to build the request to fetch IPv6 CIDRs")?;
        let v6_resp =
            client
                .execute(v6_req).await
                .and_then(|r| r.error_for_status())
                .context("Request to fetch IPv6 CIDRs failed")?
                .text().await
                .context("Failed to get the IPv6 response body")?;
        let v6_nets = read_lines::<Ipv6Net>(&v6_resp);

        // get existing routes in the table
        let routes =
            handle.route()
                .get(V6)
                .execute()
                .try_filter(|x| ready(x.header.table == arg.table));

        // delete existing routes
        routes.try_for_each_concurrent(
            10,
            |r| handle.route().del(r).execute(),
        ).await.context("Failed to delete route")?;

        let mut v6_count = 0u64;

        // add the routes
        iter(v6_nets)
            .map(Ok)
            .try_for_each_concurrent(
                10,
                |n| {
                    v6_count += 1;
                    handle.route()
                        .add()
                        .v6()
                        .table(arg.table)
                        .output_interface(if_id)
                        .destination_prefix(n.addr(), n.prefix_len())
                        .execute()
                },
            ).await
            .context("Failed to add route")?;

        println!("Successfully added {v4_count} IPv4 routes and {v6_count} IPv6 routes.");
    } else {
        println!("Successfully added {v4_count} IPv4 routes.");
    }
    Ok(())
}

fn try_notify_systemd() -> Result<()> {
    if env::var("INVOCATION_ID").is_ok() {
        notify(false, [("READY", "1")].iter())
            .context("Failed to notify systemd")?;
    }
    Ok(())
}
