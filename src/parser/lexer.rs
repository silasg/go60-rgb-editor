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
        // Arrange
        let hex_color_input = "0xFF0000";

        // Act
        let (remaining_input, parsed_hex) = hex_number(hex_color_input).unwrap();

        // Assert
        assert_eq!(parsed_hex, "0xFF0000");
        assert_eq!(remaining_input, "");
    }

    #[test]
    fn test_define_rgb() {
        // Arrange
        let rgb_define_line = "#define RED_RGB 0xFF0000 // Red color";

        // Act
        let (_, (name, hex, comment)) = define_rgb(rgb_define_line).unwrap();

        // Assert
        assert_eq!(name, "RED_RGB");
        assert_eq!(hex, "0xFF0000");
        assert_eq!(comment, Some(" Red color"));
    }

    #[test]
    fn test_define_underglow() {
        // Arrange
        let underglow_define_line = "  #define RED &ug RED_RGB";

        // Act
        let (_, (color_name, rgb_reference)) = define_underglow(underglow_define_line).unwrap();

        // Assert
        assert_eq!(color_name, "RED");
        assert_eq!(rgb_reference, "RED_RGB");
    }

    #[test]
    fn test_define_lock_indicator() {
        // Arrange
        let lock_indicator_line = "#define BSL &ug_sl BLK_RGB RED_RGB // comment";

        // Act
        let (_, (name, indicator_type, off_color, on_color)) =
            define_lock_indicator(lock_indicator_line).unwrap();

        // Assert
        assert_eq!(name, "BSL");
        assert_eq!(indicator_type, "&ug_sl");
        assert_eq!(off_color, "BLK_RGB");
        assert_eq!(on_color, "RED_RGB");
    }

    #[test]
    fn test_ifdef_layer() {
        // Arrange
        let ifdef_line = "      #ifdef LAYER_Cursor";

        // Act
        let (_, layer_macro_name) = ifdef_layer(ifdef_line).unwrap();

        // Assert
        assert_eq!(layer_macro_name, "LAYER_Cursor");
    }

    #[test]
    fn test_identifier_parses_alphanumeric_and_underscores() {
        // Arrange
        let input_with_trailing = "RED_RGB rest";

        // Act
        let (remaining, parsed_ident) = identifier(input_with_trailing).unwrap();

        // Assert
        assert_eq!(parsed_ident, "RED_RGB", "should parse up to first non-ident character");
        assert_eq!(remaining, " rest", "should leave the rest of the input");
    }

    #[test]
    fn test_line_comment_extracts_comment_text() {
        // Arrange
        let input_with_comment = "  // This is a comment";

        // Act
        let (_, comment_text) = line_comment(input_with_comment).unwrap();

        // Assert
        assert_eq!(comment_text, " This is a comment");
    }

    #[test]
    fn test_opt_line_comment_returns_none_when_absent() {
        // Arrange
        let input_without_comment = "   ";

        // Act
        let (_, comment) = opt_line_comment(input_without_comment).unwrap();

        // Assert
        assert_eq!(comment, None, "should return None when no comment is present");
    }

    #[test]
    fn test_ws_skips_whitespace_and_newlines() {
        // Arrange
        let input_with_whitespace = "  \n\t remaining";

        // Act
        let (remaining, skipped) = ws(input_with_whitespace).unwrap();

        // Assert
        assert_eq!(skipped, "  \n\t ", "should consume all whitespace characters");
        assert_eq!(remaining, "remaining");
    }

    #[test]
    fn test_define_alias_parses_simple_alias() {
        // Arrange
        let alias_line = "#define FST GOL // fast color";

        // Act
        let (_, (alias_name, target_name)) = define_alias(alias_line).unwrap();

        // Assert
        assert_eq!(alias_name, "FST", "should parse the alias name");
        assert_eq!(target_name, "GOL", "should parse the target name");
    }

    #[test]
    fn test_define_alias_rejects_underglow_binding() {
        // Arrange
        let underglow_line = "#define RED &ug RED_RGB";

        // Act
        let result = define_alias(underglow_line);

        // Assert
        assert!(result.is_err(), "define_alias should reject lines that start with &ug");
    }

    #[test]
    fn test_endif_parses_endif_directive() {
        // Arrange
        let endif_line = "  #endif";

        // Act
        let result = endif(endif_line);

        // Assert
        assert!(result.is_ok(), "should parse #endif with leading whitespace");
    }

    #[test]
    fn test_layer_id_parses_layer_identifier() {
        // Arrange
        let layer_id_line = "        layer-id = <LAYER_Cursor>;";

        // Act
        let (_, parsed_layer_id) = layer_id(layer_id_line).unwrap();

        // Assert
        assert_eq!(parsed_layer_id, "LAYER_Cursor");
    }

    #[test]
    fn test_fade_delay_parses_numeric_value() {
        // Arrange
        let fade_delay_line = "        fade-delay = <30>;";

        // Act
        let (_, parsed_delay) = fade_delay(fade_delay_line).unwrap();

        // Assert
        assert_eq!(parsed_delay, 30, "should parse the fade delay value as u16");
    }

    #[test]
    fn test_define_rgb_without_comment() {
        // Arrange
        let rgb_line_no_comment = "#define GRN_RGB 0x00FF00";

        // Act
        let (_, (name, hex, comment)) = define_rgb(rgb_line_no_comment).unwrap();

        // Assert
        assert_eq!(name, "GRN_RGB");
        assert_eq!(hex, "0x00FF00");
        assert_eq!(comment, None, "should return None when no comment is present");
    }

    #[test]
    fn test_define_lock_indicator_numlock_variant() {
        // Arrange
        let numlock_line = "#define BNL &ug_nl BLK_RGB GRN_RGB";

        // Act
        let (_, (name, indicator_type, off_color, on_color)) =
            define_lock_indicator(numlock_line).unwrap();

        // Assert
        assert_eq!(name, "BNL");
        assert_eq!(indicator_type, "&ug_nl", "should parse &ug_nl variant");
        assert_eq!(off_color, "BLK_RGB");
        assert_eq!(on_color, "GRN_RGB");
    }

    #[test]
    fn test_define_lock_indicator_capslock_variant() {
        // Arrange
        let capslock_line = "#define BCL &ug_cl BLK_RGB BLU_RGB";

        // Act
        let (_, (name, indicator_type, off_color, on_color)) =
            define_lock_indicator(capslock_line).unwrap();

        // Assert
        assert_eq!(name, "BCL");
        assert_eq!(indicator_type, "&ug_cl", "should parse &ug_cl variant");
        assert_eq!(off_color, "BLK_RGB");
        assert_eq!(on_color, "BLU_RGB");
    }
}
