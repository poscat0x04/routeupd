pub use nom::{IResult, Err};
use nom::multi::many0;
use nom::sequence::tuple;
use nom::bytes::complete::{take_while, take_till};
use nom::character::complete::char;
use nom::branch::alt;

pub use std::net::{Ipv4Addr, Ipv6Addr};

#[derive(Debug, Eq, PartialEq)]
pub struct Ipv4Net {
    pub addr: Ipv4Addr,
    pub prefix: u8,
}

#[derive(Debug, Eq, PartialEq)]
pub struct Ipv6Net {
    pub addr: Ipv6Addr,
    pub prefix: u8,
}

pub type Res<'a, T> = IResult<&'a str, T, ()>;

fn ip_prefix(max: u8, input: &str) -> Res<u8> {
    todo!()
}

fn ipv4_segment(input: &str) -> Res<u8> {
    todo!()
}

fn ipv6_segment(input: &str) -> Res<u8> {
    todo!()
}

fn single_ipv4_addr(input: &str) -> Res<Ipv4Net> {
    let (input, ()) = lex_ws(input)?;
    let (input, (a, _, b, _, c, _, d, _, prefix)) = tuple((
        ipv4_segment,
        char('.'),
        ipv4_segment,
        char('.'),
        ipv4_segment,
        char('.'),
        ipv4_segment,
        char('/'),
        |i| ip_prefix(32, i),
    ))(input)?;
    Ok((input, Ipv4Net{
        addr: Ipv4Addr::new(a, b, c, d),
        prefix
    }))
}

fn single_ipv6_addr(input: &str) -> Res<Ipv6Net> {
    todo!()
}

pub fn many_ip_addr(input: &str) -> Res<Vec<Ipv4Net>> {
    many0(single_ipv4_addr)(input)
}

pub fn lex_ws(input: &str) -> Res<()> {
    let (input, _) = many0(alt((
        comment,
        lex_space
    )))(input)?;
    Ok((input, ()))
}

pub fn lex_space(input: &str) -> Res<()> {
    let (input, _) = take_while(|c: char| c.is_whitespace())(input)?;
    Ok((input, ()))
}

pub fn comment(input: &str) -> Res<()> {
    let (input, _) = tuple((
        char('#'),
        take_till(|c| c == '\n'),
        char('\n'),
    ))(input)?;
    Ok((input, ()))
}

pub fn file(input: &str) -> Res<Vec<Ipv4Net>> {
    let (input, (v, _)) = tuple((
       many_ip_addr,
       lex_ws
    ))(input)?;
    Ok((input, v))
}

#[cfg(test)]
mod test {
    use crate::parser::*;

    #[test]
    fn parse_comment() {
    }

    #[test]
    fn parse_multiline_whitespace() {
    }

    #[test]
    fn parse_ipv4_cidr() {
        assert_eq!(
            single_ipv4_addr("1.1.1.0/24"),
            Ok(("",Ipv4Net{
                addr: Ipv4Addr::new(1,1,1,0),
                prefix: 24
            }))
        );
    }

    #[test]
    fn parse_ipv6_cidr() {
    }
}
