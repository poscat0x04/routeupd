use argh::FromArgs;

const V4_DEFAULT_URL: &str = "https://raw.githubusercontent.com/misakaio/chnroutes2/master/chnroutes.txt";
const V6_DEFAULT_URL: &str = "https://raw.githubusercontent.com/fernvenue/chn-cidr-list/master/ipv6.txt";

#[derive(FromArgs, Debug)]
/// Routing table updating tool
pub struct Args {
    /// URL to the IPv4 CIDR list.
    /// Default: https://raw.githubusercontent.com/misakaio/chnroutes2/master/chnroutes.txt
    #[argh(option, short = '4', long = "v4-url", default = "String::from(V4_DEFAULT_URL)")]
    pub v4_url: String,
    /// URL to the IPv6 CIDR list
    /// Default: https://raw.githubusercontent.com/fernvenue/chn-cidr-list/master/ipv6.txt
    #[argh(option, short = '6', long = "v6-url", default = "String::from(V6_DEFAULT_URL)")]
    pub v6_url: String,
    /// whether to add IPv6 routes
    #[argh(switch, long = "no-v6")]
    pub no_v6: bool,
    /// whether to start routeupd in daemon mode
    #[argh(switch, short = 'd', long = "daemon")]
    pub daemon: bool,
    /// the interval between update, defaults to 1 day, uses systemd.time(7) syntax
    #[argh(option, long = "interval")]
    pub interval: Option<String>,
    /// the output interface of the routes
    #[argh(option, short = 'i', long = "interface")]
    pub interface: String,
    /// the id of the routing table to add routes to, should be exclusively managed by routeupd
    #[argh(option, short = 't', long = "table")]
    pub table: u8,
}
