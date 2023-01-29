mod parser;
mod cli;
mod url;

use std::time::Duration;
use std::env;
use std::future::ready;

use anyhow::{Result, Context, anyhow};
use futures_util::stream::iter;
use futures_util::{StreamExt, TryStreamExt};
use ipnet::{Ipv4Net, Ipv6Net};
use parse_duration::parse::parse;
use reqwest::{Client, ClientBuilder, Url};
use rtnetlink::{new_connection, Handle};
use nix::unistd::geteuid;
use rtnetlink::IpVersion::{V4, V6};
use systemd::daemon::notify;
use tokio::time::sleep;

use cli::{Arg, Parser};
use parser::read_lines;
use crate::url::parse_url;

const UPDATE_INTERVAL_DEFAULT: Duration = Duration::from_secs(24 * 60 * 60);

#[tokio::main]
async fn main() -> Result<()> {
    // check if this program have root privilege
    // immediately exits if does not
    if !geteuid().is_root() {
        eprintln!("routeupd needs root privilege to use netlink to modify routing tables!");
        return Err(anyhow!("Not enough privilege"))
    }

    // parse arguments
    let arg = Arg::parse();

    // parse interval string, if supplied
    let update_interval = match &arg.interval {
        Some(i) => parse(i).with_context(|| format!("Failed to parse interval \"{}\"", i))?,
        None => UPDATE_INTERVAL_DEFAULT,
    };

    // establish netlink connection
    let (conn, handle, _) =
        new_connection()
            .context("Failed to establish a netlink connection")?;
    tokio::spawn(conn);

    // get the interface id
    let interface =
        handle.link()
            .get()
            .match_name(arg.interface.clone())
            .execute()
            .try_next().await
            .with_context(||
                format!("Failed to get the id of the interface with the name {}", arg.interface)
            )?
            .with_context(||
                format!("No interface with the name {} was found", arg.interface)
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

    let v4_url = parse_url(&arg.v4_url).context("Failed to parse URL")?;
    let v6_url = parse_url(&arg.v6_url).context("Failed to parse URL")?;

    update_routes(
        &client, &handle, &arg,
        if_id, &v4_url, &v6_url,
    ).await?;
    try_notify_systemd()?;

    if arg.daemon {
        loop {
            sleep(update_interval).await;
            update_routes(
                &client, &handle, &arg,
                if_id, &v4_url, &v6_url,
            ).await?
        }
    }

    Ok(())
}

async fn update_routes(
    client: &Client,
    handle: &Handle,
    arg: &Arg,
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
        |r| handle.route().del(r).execute()
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
            }
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
            |r| handle.route().del(r).execute()
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
                }
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
