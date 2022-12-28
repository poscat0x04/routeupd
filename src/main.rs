mod parser;
mod cli;

use rtnetlink::{new_connection, Handle};
use anyhow::{Result, Context};

#[tokio::main]
async fn main() -> Result<()> {
    let (conn, handle, _) = new_connection().context("")?;
    tokio::spawn(conn);

    Ok(())
}
