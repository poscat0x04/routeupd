use reqwest::Client;
use rtnetlink::Handle;
use url::Url;
use ipnet::{Ipv4Net, Ipv6Net};
use rtnetlink::IpVersion::{V4, V6};
use std::future::ready;
use futures_util::stream::iter;
use anyhow::Context;
use futures_util::{StreamExt, TryStreamExt};
use crate::cli::Args;
use crate::parser::read_lines;

pub async fn update_routes(
    client: &Client,
    handle: &Handle,
    arg: &Args,
    if_id: u32,
    v4_url: &Url,
    v6_url: &Url,
) -> anyhow::Result<()> {

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
