//! Lexer utilities for parsing ZMK config files.
//! These nom combinators are available for future use but the current
//! parser uses simpler string operations in grammar.rs.

#![allow(dead_code)]

use nom::{
    IResult,
    bytes::complete::{tag, take_while1, take_while},
    character::complete::{space0, space1, not_line_ending},
    combinator::{opt, recognize},
    sequence::{preceded, tuple},
    branch::alt,
};

/// Identifier characters (alphanumeric + underscore)
fn is_ident_char(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

/// Parse an identifier
pub fn identifier(input: &str) -> IResult<&str, &str> {
    take_while1(is_ident_char)(input)
}

/// Parse a hex number (0xNNNNNN)
pub fn hex_number(input: &str) -> IResult<&str, &str> {
    recognize(tuple((
        tag("0x"),
        take_while1(|c: char| c.is_ascii_hexdigit()),
    )))(input)
}

/// Parse rest of line as comment (after //)
pub fn line_comment(input: &str) -> IResult<&str, &str> {
    preceded(
        tuple((space0, tag("//"))),
        not_line_ending,
    )(input)
}

/// Parse optional comment at end of line
pub fn opt_line_comment(input: &str) -> IResult<&str, Option<&str>> {
    opt(line_comment)(input)
}

/// Skip whitespace and newlines
pub fn ws(input: &str) -> IResult<&str, &str> {
    take_while(|c: char| c.is_whitespace())(input)
}

/// Parse a #define line for RGB color: #define NAME_RGB 0xNNNNNN // comment
pub fn define_rgb(input: &str) -> IResult<&str, (&str, &str, Option<&str>)> {
    let (input, _) = tuple((space0, tag("#define"), space1))(input)?;
    let (input, name) = identifier(input)?;
    let (input, _) = space1(input)?;
    let (input, hex) = hex_number(input)?;
    let (input, comment) = opt_line_comment(input)?;
    
    Ok((input, (name, hex, comment)))
}

/// Parse a #define line for underglow binding: #define NAME &ug NAME_RGB
pub fn define_underglow(input: &str) -> IResult<&str, (&str, &str)> {
    let (input, _) = tuple((space0, tag("#define"), space1))(input)?;
    let (input, name) = identifier(input)?;
    let (input, _) = tuple((space1, tag("&ug"), space1))(input)?;
    let (input, rgb_name) = identifier(input)?;
    let (input, _) = opt_line_comment(input)?;
    
    Ok((input, (name, rgb_name)))
}

/// Parse a #define line for lock indicator: #define NAME &ug_sl OFF_RGB ON_RGB
/// or &ug_nl, &ug_cl variants
pub fn define_lock_indicator(input: &str) -> IResult<&str, (&str, &str, &str, &str)> {
    let (input, _) = tuple((space0, tag("#define"), space1))(input)?;
    let (input, name) = identifier(input)?;
    let (input, _) = space1(input)?;
    let (input, ug_type) = alt((tag("&ug_sl"), tag("&ug_nl"), tag("&ug_cl")))(input)?;
    let (input, _) = space1(input)?;
    let (input, off_rgb) = identifier(input)?;
    let (input, _) = space1(input)?;
    let (input, on_rgb) = identifier(input)?;
    let (input, _) = opt_line_comment(input)?;
    
    Ok((input, (name, ug_type, off_rgb, on_rgb)))
}

/// Parse a #define alias: #define NAME TARGET (where TARGET is not &ug)
pub fn define_alias(input: &str) -> IResult<&str, (&str, &str)> {
    let (input, _) = tuple((space0, tag("#define"), space1))(input)?;
    let (input, name) = identifier(input)?;
    let (input, _) = space1(input)?;
    // Make sure it's not &ug
    if input.starts_with("&ug") {
        return Err(nom::Err::Error(nom::error::Error::new(input, nom::error::ErrorKind::Tag)));
    }
    let (input, target) = identifier(input)?;
    let (input, _) = opt_line_comment(input)?;
    
    Ok((input, (name, target)))
}

/// Parse #ifdef LAYER_Name
pub fn ifdef_layer(input: &str) -> IResult<&str, &str> {
    let (input, _) = tuple((space0, tag("#ifdef"), space1))(input)?;
    let (input, name) = identifier(input)?;
    Ok((input, name))
}

/// Parse #endif
pub fn endif(input: &str) -> IResult<&str, ()> {
    let (input, _) = tuple((space0, tag("#endif")))(input)?;
    Ok((input, ()))
}

/// Parse layer-id line: layer-id = <LAYER_Name>;
pub fn layer_id(input: &str) -> IResult<&str, &str> {
    let (input, _) = tuple((space0, tag("layer-id"), space0, tag("="), space0, tag("<")))(input)?;
    let (input, name) = identifier(input)?;
    let (input, _) = tuple((tag(">"), space0, tag(";")))(input)?;
    Ok((input, name))
}

/// Parse fade-delay line: fade-delay = <N>;
pub fn fade_delay(input: &str) -> IResult<&str, u16> {
    let (input, _) = tuple((space0, tag("fade-delay"), space0, tag("="), space0, tag("<")))(input)?;
    let (input, num_str) = take_while1(|c: char| c.is_ascii_digit())(input)?;
    let (input, _) = tuple((tag(">"), space0, tag(";")))(input)?;
    let num = num_str.parse().unwrap_or(30);
    Ok((input, num))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hex_number() {
        let (rest, hex) = hex_number("0xFF0000").unwrap();
        assert_eq!(hex, "0xFF0000");
        assert_eq!(rest, "");
    }

    #[test]
    fn test_define_rgb() {
        let (_, (name, hex, comment)) = define_rgb("#define RED_RGB 0xFF0000 // Red color").unwrap();
        assert_eq!(name, "RED_RGB");
        assert_eq!(hex, "0xFF0000");
        assert_eq!(comment, Some(" Red color"));
    }

    #[test]
    fn test_define_underglow() {
        let (_, (name, rgb)) = define_underglow("  #define RED &ug RED_RGB").unwrap();
        assert_eq!(name, "RED");
        assert_eq!(rgb, "RED_RGB");
    }

    #[test]
    fn test_define_lock_indicator() {
        let (_, (name, ug_type, off, on)) = define_lock_indicator("#define BSL &ug_sl BLK_RGB RED_RGB // comment").unwrap();
        assert_eq!(name, "BSL");
        assert_eq!(ug_type, "&ug_sl");
        assert_eq!(off, "BLK_RGB");
        assert_eq!(on, "RED_RGB");
    }

    #[test]
    fn test_ifdef_layer() {
        let (_, name) = ifdef_layer("      #ifdef LAYER_Cursor").unwrap();
        assert_eq!(name, "LAYER_Cursor");
    }
}
