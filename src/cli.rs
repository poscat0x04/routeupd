pub use clap::Parser;

const V4_DEFAULT_URL: &str = "";
const V6_DEFAULT_URL: &str = "";

#[derive(Parser)]
#[command(author, version, about)]
pub struct Arg {
    #[arg(short = '4', long = "v4-url", default_value_t = String::from(V4_DEFAULT_URL))]
    pub v4_url: String,
    #[arg(short = '6', long = "v6-url", default_value_t = String::from(V6_DEFAULT_URL))]
    pub v6_url: String,
    #[arg(long = "no-v6", default_value_t = false)]
    pub no_v6: bool,
    #[arg(short, long = "daemon", default_value_t = false)]
    pub daemon: bool,
    #[arg(short = 'i', long = "interface")]
    pub interface: String,
    #[arg(short = 't', long = "table")]
    pub table: u8
}
