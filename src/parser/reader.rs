use crate::model::{Config, ColorDef, ColorKind, ColorPalette, Layer, RgbColor};
use std::path::PathBuf;

pub fn parse_config(input: &str) -> Result<Config, String> {
    let mut config = Config::new(PathBuf::new());

    // Find the underglow-layer section markers
    let section_start = "// ==== PER-KEY-RGB <section begins> ====";
    let section_end = "// ==== PER-KEY-RGB <section ends> =====";

    let start_pos = input
        .find(section_start)
        .ok_or("Could not find PER-KEY-RGB section start marker")?;
    let end_pos = input
        .find(section_end)
        .ok_or("Could not find PER-KEY-RGB section end marker")?;

    // Extract header (everything before section start)
    config.raw_header = input[..start_pos].to_string();

    // Extract footer (everything after section end marker line)
    let footer_start = input[end_pos..]
        .find('\n')
        .map(|i| end_pos + i + 1)
        .unwrap_or(input.len());
    config.raw_footer = input[footer_start..].to_string();

    // Parse color palette from header
    config.palette = parse_palette(&config.raw_header)?;

    // Parse layers from section
    let section = &input[start_pos..end_pos];
    config.layers = parse_layers(section)?;

    Ok(config)
}

fn parse_palette(header: &str) -> Result<ColorPalette, String> {
    let mut palette = ColorPalette::new();
    let mut rgb_colors: std::collections::HashMap<String, (RgbColor, Option<String>)> =
        std::collections::HashMap::new();

    // First pass: collect all _RGB color definitions
    for line in header.lines() {
        let line = line.trim();
        if !line.starts_with("#define") {
            continue;
        }

        // Try to parse RGB definition: #define NAME_RGB 0xNNNNNN // comment
        if let Some(rgb_def) = parse_rgb_define(line) {
            rgb_colors.insert(rgb_def.name.to_string(), (rgb_def.rgb, rgb_def.comment));
        }
    }

    // Second pass: collect underglow bindings and build palette
    for line in header.lines() {
        let line = line.trim();
        if !line.starts_with("#define") {
            continue;
        }

        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 3 {
            continue;
        }
        if parts[1].ends_with("_RGB") {
            continue;
        }

        let define = DefineLine {
            name: parts[1],
            binding: parts[2],
            arg1: parts.get(3).copied(),
            arg2: parts.get(4).copied(),
        };

        if let Some(color_def) = parse_underglow_binding(&define, &rgb_colors)
            .or_else(|| parse_lock_indicator(&define, &rgb_colors))
            .or_else(|| parse_alias(&define, &palette))
        {
            palette.add(color_def);
        }
    }

    Ok(palette)
}

/// A parsed `#define NAME BINDING [ARG1] [ARG2]` line.
struct DefineLine<'a> {
    name: &'a str,
    binding: &'a str,
    arg1: Option<&'a str>,
    arg2: Option<&'a str>,
}

fn parse_underglow_binding(
    define: &DefineLine,
    rgb_colors: &std::collections::HashMap<String, (RgbColor, Option<String>)>,
) -> Option<ColorDef> {
    if define.binding != "&ug" {
        return None;
    }
    let rgb_name = define.arg1?;
    let (rgb, comment) = rgb_colors.get(rgb_name)?;
    let mut color_def = ColorDef::new(define.name.to_string(), rgb.clone());
    if let Some(c) = comment {
        color_def = color_def.with_comment(c.clone());
    }
    Some(color_def)
}

fn parse_lock_indicator(
    define: &DefineLine,
    rgb_colors: &std::collections::HashMap<String, (RgbColor, Option<String>)>,
) -> Option<ColorDef> {
    if !matches!(define.binding, "&ug_sl" | "&ug_nl" | "&ug_cl") {
        return None;
    }
    let off_rgb_name = define.arg1?;
    let on_rgb_name = define.arg2?;

    let rgb = rgb_colors
        .get(on_rgb_name)
        .map(|(r, _)| r.clone())
        .unwrap_or_default();

    let off_abbrev = off_rgb_name.trim_end_matches("_RGB").to_string();
    let on_abbrev = on_rgb_name.trim_end_matches("_RGB").to_string();

    Some(
        ColorDef::new(define.name.to_string(), rgb).with_kind(ColorKind::LockIndicator {
            off_color: off_abbrev,
            on_color: on_abbrev,
        }),
    )
}

fn parse_alias(define: &DefineLine, palette: &ColorPalette) -> Option<ColorDef> {
    if define.binding.starts_with('&') || define.binding.starts_with("0x") {
        return None;
    }
    let target = define.binding;
    let rgb = palette
        .get(target)
        .map(|c| c.rgb.clone())
        .unwrap_or_default();

    Some(
        ColorDef::new(define.name.to_string(), rgb).with_kind(ColorKind::Alias {
            target: target.to_string(),
        }),
    )
}

struct RgbDefinition<'a> {
    name: &'a str,
    rgb: RgbColor,
    comment: Option<String>,
}

fn parse_rgb_define(line: &str) -> Option<RgbDefinition<'_>> {
    let parts: Vec<&str> = line.splitn(4, ' ').collect();
    if parts.len() < 3 || parts[0] != "#define" {
        return None;
    }

    let name = parts[1];
    if !name.ends_with("_RGB") {
        return None;
    }

    let hex_str = parts[2];
    let rgb = RgbColor::from_hex(hex_str).ok()?;

    let comment = if parts.len() > 3 {
        let rest = parts[3];
        rest.find("//").map(|pos| rest[pos + 2..].trim().to_string())
    } else {
        None
    };

    Some(RgbDefinition { name, rgb, comment })
}

enum LayerParseState {
    /// Outside any #ifdef block
    Idle,
    /// Inside a layer block, reading properties
    InLayer,
    /// Inside a `bindings = < ... >;` block
    InBindings(String),
}

fn parse_layers(section: &str) -> Result<Vec<Layer>, String> {
    let mut layers = Vec::new();
    let mut current_layer: Option<Layer> = None;
    let mut state = LayerParseState::Idle;

    for line in section.lines() {
        let trimmed = line.trim();

        if trimmed.starts_with("#ifdef LAYER_") {
            let macro_name = trimmed
                .strip_prefix("#ifdef ")
                .unwrap_or("")
                .trim()
                .to_string();
            let name = macro_name
                .strip_prefix("LAYER_")
                .unwrap_or(&macro_name)
                .to_string();
            current_layer = Some(Layer::new(name, macro_name));
            state = LayerParseState::InLayer;
            continue;
        }

        if trimmed == "#endif" {
            if let Some(layer) = current_layer.take() {
                layers.push(layer);
            }
            state = LayerParseState::Idle;
            continue;
        }

        if let Some(ref mut layer) = current_layer {
            match &mut state {
                LayerParseState::InBindings(ref mut content) => {
                    if trimmed.starts_with(">;") {
                        parse_bindings(content, layer)?;
                        state = LayerParseState::InLayer;
                    } else {
                        content.push_str(line);
                        content.push('\n');
                    }
                }
                LayerParseState::InLayer => {
                    if trimmed.starts_with("bindings = <") {
                        state = LayerParseState::InBindings(String::new());
                    } else if trimmed.starts_with("layer-id") {
                        // Already captured from #ifdef
                    } else if trimmed.starts_with("fade-delay") {
                        if let (Some(start), Some(end)) = (trimmed.find('<'), trimmed.find('>')) {
                            if let Ok(delay) = trimmed[start + 1..end].parse() {
                                layer.fade_delay = delay;
                            }
                        }
                    }
                }
                LayerParseState::Idle => {}
            }
        }
    }

    Ok(layers)
}

/// Parse bindings into left/right halves.
/// Main rows have 12 tokens (6+6), thumb rows have 6 tokens (3+3).
fn parse_bindings(content: &str, layer: &mut Layer) -> Result<(), String> {
    let mut left_half: Vec<Vec<String>> = Vec::new();
    let mut right_half: Vec<Vec<String>> = Vec::new();

    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        let tokens: Vec<&str> = line.split_whitespace().collect();
        if tokens.is_empty() {
            continue;
        }

        // Determine if this is a main row (6+6 keys) or thumb row (3+3 keys)
        if tokens.len() == 12 {
            left_half.push(tokens[0..6].iter().map(|s| s.to_string()).collect());
            right_half.push(tokens[6..12].iter().map(|s| s.to_string()).collect());
        } else if tokens.len() == 6 {
            left_half.push(tokens[0..3].iter().map(|s| s.to_string()).collect());
            right_half.push(tokens[3..6].iter().map(|s| s.to_string()).collect());
        } else {
            return Err(format!(
                "Unexpected token count {} in bindings line: '{}'",
                tokens.len(), line
            ));
        }
    }

    layer.left_half = left_half;
    layer.right_half = right_half;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_rgb_define() {
        // Arrange
        let rgb_define_line = "#define RED_RGB 0xFF0000 // Red color";

        // Act
        let result = parse_rgb_define(rgb_define_line);

        // Assert
        assert!(result.is_some());
        let def = result.unwrap();
        assert_eq!(def.name, "RED_RGB");
        assert_eq!(def.rgb, RgbColor::new(255, 0, 0));
        assert_eq!(def.comment, Some("Red color".to_string()));
    }

    #[test]
    fn test_parse_bindings() {
        // Arrange
        let bindings_content = r#"
          ___ ___ BCL BNL BSL ___                                     ___ ___ ___ ___ ___ ___
          ___ ___ ___ ___ ___ ___                                     ___ ___ ___ ___ ___ ___
          ___ MAJ CYN CHU YLW ___                                     ___ YLW CHU CYN MAJ ___
          ___ ___ ___ ___ ___ ___                                     ___ ___ ___ ___ ___ ___
                  ___ ___ ___                                             ___ ___ ___ 
                                       ___ ___ ___     ___ ___ ___ 
        "#;
        let mut layer = Layer::new("Test".to_string(), "LAYER_Test".to_string());

        // Act
        parse_bindings(bindings_content, &mut layer).unwrap();

        // Assert
        let expected_row_count = 6;
        assert_eq!(layer.left_half.len(), expected_row_count);
        assert_eq!(layer.right_half.len(), expected_row_count);
        assert_eq!(layer.left_half[0][2], "BCL");
        assert_eq!(layer.left_half[2][1], "MAJ");
        assert_eq!(layer.right_half[2][4], "MAJ");
    }
}
