use clap::Parser;

#[derive(Parser)]
pub struct Arg {
    pub v4_url: String,
    pub v6_url: String,
    pub no_v6: bool,
    pub interface: String,
    pub table: u64
}
