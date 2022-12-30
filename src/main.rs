mod parser;
mod cli;

use rtnetlink::{new_connection, Handle};
use anyhow::{Result, Context};
use cli::*;

#[tokio::main]
async fn main() -> Result<()> {
    let arg = Arg::parse();
    let (conn, handle, _) = new_connection().context("")?;
    tokio::spawn(conn);

    Ok(())
}
