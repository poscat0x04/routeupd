pub use clap::Parser;

const V4_DEFAULT_URL: &str = "https://raw.githubusercontent.com/misakaio/chnroutes2/master/chnroutes.txt";
const V6_DEFAULT_URL: &str = "https://raw.githubusercontent.com/fernvenue/chn-cidr-list/master/ipv6.txt";

#[derive(Parser)]
#[command(author, version, about)]
pub struct Arg {
    #[arg(
        short = '4',
        long = "v4-url",
        default_value_t = String::from(V4_DEFAULT_URL),
        help = "URL to the IPv4 CIDR list"
    )]
    pub v4_url: String,
    #[arg(
        short = '6',
        long = "v6-url",
        default_value_t = String::from(V6_DEFAULT_URL),
        help = "URL to the IPv6 CIDR list"
    )]
    pub v6_url: String,
    #[arg(
        long = "no-v6",
        default_value_t = false,
        help = "Whether to add IPv6 routes"
    )]
    pub no_v6: bool,
    #[arg(
        short,
        long = "daemon",
        default_value_t = false,
        help = "Whether to start routeupd in daemon mode"
    )]
    pub daemon: bool,
    #[arg(
        short = 'i',
        long = "interface",
        help = "The output interface"
    )]
    pub interface: String,
    #[arg(
        short = 't',
        long = "table",
        help = "The id of the routing table to add routes to, should be exclusively managed by routeupd"
    )]
    pub table: u8,
    #[arg(
        long = "interval",
        help = "The interval between update, defaults to 1 day, uses systemd.time(7) syntax"
    )]
    pub interval: Option<String>,
}
