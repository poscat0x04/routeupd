use std::str::FromStr;

fn read_lines<T: FromStr>(f: &str) -> impl Iterator<Item = T> + '_
{
    f
        .lines()
        .map(|l| T::from_str(l.trim()))
        .filter_map(|i| i.ok())
}

#[cfg(test)]
mod test {
    use std::str::FromStr;
    use ipnet::{Ipv4Net, Ipv6Net};

    use crate::parser::read_lines;

    #[test]
    fn test_parse_ipv4() {
        const TEST_FILE: &str = r#"
            1.1.1.1/32
            2.2.3.4/8
            3.4.5.6/32
        "#;
        let mut it = read_lines::<Ipv4Net>(TEST_FILE);
        assert_eq!(it.next(), Ipv4Net::from_str("1.1.1.1/32").ok());
        assert_eq!(it.next(), Ipv4Net::from_str("2.2.3.4/8").ok());
        assert_eq!(it.next(), Ipv4Net::from_str("3.4.5.6/32").ok());
        assert_eq!(it.next(), None);
    }

    #[test]
    fn test_parse_ipv6() {
        const TEST_FILE: &str = r#"
            2001:cc0::/32
            2001:df1:2b40::/48
        "#;
        let mut it = read_lines::<Ipv6Net>(TEST_FILE);
        assert_eq!(it.next(), Ipv6Net::from_str("2001:cc0:0::/32").ok());
        assert_eq!(it.next(), Ipv6Net::from_str("2001:df1:2b40::/48").ok());
        assert_eq!(it.next(), None);
    }
}
