use crate::domain::Config;

pub fn write_config(config: &Config) -> String {
    let mut output = String::new();

    output.push_str(&config.raw_header);

    output.push_str("// ==== PER-KEY-RGB <section begins> ====\n");
    output.push_str("  / {\n");
    output.push_str("    underglow-layer {\n");
    output.push_str("      compatible = \"zmk,underglow-layer\";\n\n");

    for layer in &config.layers {
        write_layer(&mut output, layer);
    }

    output.push_str("    };\n");
    output.push_str("  };\n");
    output.push_str("  // ==== PER-KEY-RGB <section ends> =====\n");

    output.push_str(&config.raw_footer);

    output
}

fn format_key_row(keys: &[String]) -> String {
    keys.iter()
        .map(|c| format!("{:>3}", c))
        .collect::<Vec<_>>()
        .join(" ")
}

fn write_layer(output: &mut String, layer: &crate::domain::Layer) {
    let display_name = layer
        .macro_name
        .strip_prefix("LAYER_")
        .unwrap_or(&layer.name);

    output.push_str(&format!("      #ifdef {}\n", layer.macro_name));
    output.push_str(&format!("      {} {{\n", display_name));
    output.push_str("        bindings = <\n");

    for row_idx in 0..4 {
        if row_idx < layer.left_half.len() && row_idx < layer.right_half.len() {
            output.push_str(&format!(
                "          {}                                     {}\n",
                format_key_row(&layer.left_half[row_idx]),
                format_key_row(&layer.right_half[row_idx]),
            ));
        }
    }

    if layer.left_half.len() > 4 && layer.right_half.len() > 4 {
        output.push_str(&format!(
            "                  {}                                             {} \n",
            format_key_row(&layer.left_half[4]),
            format_key_row(&layer.right_half[4]),
        ));
    }

    if layer.left_half.len() > 5 && layer.right_half.len() > 5 {
        output.push_str(&format!(
            "                                      {}     {} \n",
            format_key_row(&layer.left_half[5]),
            format_key_row(&layer.right_half[5]),
        ));
    }

    output.push_str("        >;\n");
    output.push_str(&format!("        layer-id = <{}>;\n", layer.macro_name));
    output.push_str(&format!("        fade-delay = <{}>;\n", layer.fade_delay));
    output.push_str("      };\n");
    output.push_str("      #endif\n\n");
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::Layer;

    #[test]
    fn test_write_layer() {
        // Arrange
        let mut layer = Layer::new("Cursor".to_string(), "LAYER_Cursor".to_string());
        layer.fade_delay = 5;
        layer.left_half[0] = vec![
            "___".to_string(), "ORN".to_string(), "ORN".to_string(),
            "ORN".to_string(), "ORN".to_string(), "___".to_string(),
        ];
        let mut output = String::new();

        // Act
        write_layer(&mut output, &layer);

        // Assert
        assert!(output.contains("#ifdef LAYER_Cursor"));
        assert!(output.contains("ORN"));
        assert!(output.contains("fade-delay = <5>"));
    }
}
