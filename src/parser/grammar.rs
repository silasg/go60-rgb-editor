use crate::model::{Config, ColorDef, ColorKind, ColorPalette, Layer, RgbColor};
use std::path::PathBuf;

/// Parse the entire config file
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

    // Parse colors from header
    config.palette = parse_colors(&config.raw_header)?;

    // Parse layers from section
    let section = &input[start_pos..end_pos];
    config.layers = parse_layers(section)?;

    Ok(config)
}

/// Parse color definitions from the header section
fn parse_colors(header: &str) -> Result<ColorPalette, String> {
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
            rgb_colors.insert(rgb_def.0.to_string(), (rgb_def.1, rgb_def.2));
        }
    }

    // Second pass: collect underglow bindings and build palette
    for line in header.lines() {
        let line = line.trim();
        if !line.starts_with("#define") {
            continue;
        }

        // Skip _RGB definitions (already processed)
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 3 {
            continue;
        }
        let name = parts[1];
        if name.ends_with("_RGB") {
            continue;
        }

        // Check for &ug binding
        if parts.len() >= 4 && parts[2] == "&ug" {
            let rgb_name = parts[3];
            if let Some((rgb, comment)) = rgb_colors.get(rgb_name) {
                let mut color_def = ColorDef::new(name.to_string(), rgb.clone());
                if let Some(c) = comment {
                    color_def = color_def.with_comment(c.clone());
                }
                palette.add(color_def);
            }
        }
        // Check for lock indicator (&ug_sl, &ug_nl, &ug_cl)
        else if parts.len() >= 5
            && (parts[2] == "&ug_sl" || parts[2] == "&ug_nl" || parts[2] == "&ug_cl")
        {
            let off_rgb = parts[3];
            let on_rgb = parts[4];

            // Get the "on" color's RGB for display
            let rgb = rgb_colors
                .get(on_rgb)
                .map(|(r, _)| r.clone())
                .unwrap_or_default();

            // Extract abbrev from the RGB name (e.g., RED_RGB -> RED)
            let off_abbrev = off_rgb.trim_end_matches("_RGB").to_string();
            let on_abbrev = on_rgb.trim_end_matches("_RGB").to_string();

            let color_def = ColorDef::new(name.to_string(), rgb).with_kind(
                ColorKind::LockIndicator {
                    off_color: off_abbrev,
                    on_color: on_abbrev,
                },
            );
            palette.add(color_def);
        }
        // Check for alias (simple identifier, not &ug)
        else if parts.len() >= 3 && !parts[2].starts_with('&') && !parts[2].starts_with("0x") {
            let target = parts[2];
            // Get target's RGB for display
            let rgb = palette
                .get(target)
                .map(|c| c.rgb.clone())
                .unwrap_or_default();

            let color_def = ColorDef::new(name.to_string(), rgb)
                .with_kind(ColorKind::Alias {
                    target: target.to_string(),
                });
            palette.add(color_def);
        }
    }

    Ok(palette)
}

/// Parse a single RGB #define line
fn parse_rgb_define(line: &str) -> Option<(&str, RgbColor, Option<String>)> {
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
        if let Some(pos) = rest.find("//") {
            Some(rest[pos + 2..].trim().to_string())
        } else {
            None
        }
    } else {
        None
    };

    Some((name, rgb, comment))
}

/// Parse layers from the underglow-layer section
fn parse_layers(section: &str) -> Result<Vec<Layer>, String> {
    let mut layers = Vec::new();
    let mut current_layer: Option<Layer> = None;
    let mut in_bindings = false;
    let mut bindings_content = String::new();

    for line in section.lines() {
        let trimmed = line.trim();

        // Start of a layer block
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
            continue;
        }

        // End of a layer block
        if trimmed == "#endif" {
            if let Some(layer) = current_layer.take() {
                layers.push(layer);
            }
            continue;
        }

        // Inside a layer block
        if let Some(ref mut layer) = current_layer {
            // Start of bindings
            if trimmed.starts_with("bindings = <") {
                in_bindings = true;
                bindings_content.clear();
                continue;
            }

            // End of bindings
            if in_bindings && trimmed.starts_with(">;") {
                // Parse the accumulated bindings
                parse_bindings(&bindings_content, layer)?;
                in_bindings = false;
                continue;
            }

            // Accumulate bindings content
            if in_bindings {
                bindings_content.push_str(line);
                bindings_content.push('\n');
                continue;
            }

            // layer-id line
            if trimmed.starts_with("layer-id") {
                // Already have macro_name from #ifdef, could validate here
                continue;
            }

            // fade-delay line
            if trimmed.starts_with("fade-delay") {
                if let Some(start) = trimmed.find('<') {
                    if let Some(end) = trimmed.find('>') {
                        if let Ok(delay) = trimmed[start + 1..end].parse() {
                            layer.fade_delay = delay;
                        }
                    }
                }
                continue;
            }
        }
    }

    Ok(layers)
}

/// Parse the bindings content into left and right halves
fn parse_bindings(content: &str, layer: &mut Layer) -> Result<(), String> {
    // The format is 6 rows of keys, separated by a large gap between left and right
    // Rows 0-3: 6 keys per side (main rows)
    // Row 4: 3 keys per side (inner thumbs, indented)
    // Row 5: 3 keys per side (outer thumbs, centered with large gap)

    let mut left_half: Vec<Vec<String>> = Vec::new();
    let mut right_half: Vec<Vec<String>> = Vec::new();

    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        // Split by large whitespace gaps to separate left and right halves
        // Find all tokens
        let tokens: Vec<&str> = line.split_whitespace().collect();
        if tokens.is_empty() {
            continue;
        }

        // Determine if this is a main row (6+6 keys) or thumb row (3+3 keys)
        if tokens.len() == 12 {
            // Main row: 6 left + 6 right
            left_half.push(tokens[0..6].iter().map(|s| s.to_string()).collect());
            right_half.push(tokens[6..12].iter().map(|s| s.to_string()).collect());
        } else if tokens.len() == 6 {
            // Thumb row: 3 left + 3 right
            left_half.push(tokens[0..3].iter().map(|s| s.to_string()).collect());
            right_half.push(tokens[3..6].iter().map(|s| s.to_string()).collect());
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
        let (name, rgb, comment) = result.unwrap();
        assert_eq!(name, "RED_RGB");
        assert_eq!(rgb, RgbColor::new(255, 0, 0));
        assert_eq!(comment, Some("Red color".to_string()));
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
