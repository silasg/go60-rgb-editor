use crate::model::Config;

/// Write the config back to the original file format
pub fn write_config(config: &Config) -> String {
    let mut output = String::new();

    // Write header (contains color definitions, etc.)
    output.push_str(&config.raw_header);

    // Write section start marker
    output.push_str("// ==== PER-KEY-RGB <section begins> ====\n");
    output.push_str("  / {\n");
    output.push_str("    underglow-layer {\n");
    output.push_str("      compatible = \"zmk,underglow-layer\";\n\n");

    // Write each layer
    for layer in &config.layers {
        write_layer(&mut output, layer);
    }

    // Close the underglow-layer section
    output.push_str("    };\n");
    output.push_str("  };\n");
    output.push_str("  // ==== PER-KEY-RGB <section ends> =====\n");

    // Write footer (contains #undef statements)
    output.push_str(&config.raw_footer);

    output
}

fn write_layer(output: &mut String, layer: &crate::model::Layer) {
    // Write #ifdef guard
    output.push_str(&format!("      #ifdef {}\n", layer.macro_name));

    // Write layer name block
    let display_name = layer
        .macro_name
        .strip_prefix("LAYER_")
        .unwrap_or(&layer.name);
    output.push_str(&format!("      {} {{\n", display_name));
    output.push_str("        bindings = <\n");

    // Write main rows (0-3): 6 keys per half
    for row_idx in 0..4 {
        if row_idx < layer.left_half.len() && row_idx < layer.right_half.len() {
            let left = &layer.left_half[row_idx];
            let right = &layer.right_half[row_idx];

            // Format with proper spacing
            let left_str: String = left
                .iter()
                .map(|c| format!("{:>3}", c))
                .collect::<Vec<_>>()
                .join(" ");
            let right_str: String = right
                .iter()
                .map(|c| format!("{:>3}", c))
                .collect::<Vec<_>>()
                .join(" ");

            output.push_str(&format!(
                "          {}                                     {}\n",
                left_str, right_str
            ));
        }
    }

    // Write inner thumb row (row 4): 3 keys per half, indented
    if layer.left_half.len() > 4 && layer.right_half.len() > 4 {
        let left = &layer.left_half[4];
        let right = &layer.right_half[4];

        let left_str: String = left
            .iter()
            .map(|c| format!("{:>3}", c))
            .collect::<Vec<_>>()
            .join(" ");
        let right_str: String = right
            .iter()
            .map(|c| format!("{:>3}", c))
            .collect::<Vec<_>>()
            .join(" ");

        output.push_str(&format!(
            "                  {}                                             {} \n",
            left_str, right_str
        ));
    }

    // Write outer thumb row (row 5): 3 keys per half, centered with gap
    if layer.left_half.len() > 5 && layer.right_half.len() > 5 {
        let left = &layer.left_half[5];
        let right = &layer.right_half[5];

        let left_str: String = left
            .iter()
            .map(|c| format!("{:>3}", c))
            .collect::<Vec<_>>()
            .join(" ");
        let right_str: String = right
            .iter()
            .map(|c| format!("{:>3}", c))
            .collect::<Vec<_>>()
            .join(" ");

        output.push_str(&format!(
            "                                      {}     {} \n",
            left_str, right_str
        ));
    }

    // Close bindings
    output.push_str("        >;\n");

    // Write layer-id and fade-delay
    output.push_str(&format!("        layer-id = <{}>;\n", layer.macro_name));
    output.push_str(&format!("        fade-delay = <{}>;\n", layer.fade_delay));

    // Close layer block
    output.push_str("      };\n");
    output.push_str("      #endif\n\n");
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Layer;

    #[test]
    fn test_write_layer() {
        let mut layer = Layer::new("Cursor".to_string(), "LAYER_Cursor".to_string());
        layer.fade_delay = 5;
        layer.left_half[0] = vec![
            "___".to_string(),
            "ORN".to_string(),
            "ORN".to_string(),
            "ORN".to_string(),
            "ORN".to_string(),
            "___".to_string(),
        ];

        let mut output = String::new();
        write_layer(&mut output, &layer);

        assert!(output.contains("#ifdef LAYER_Cursor"));
        assert!(output.contains("ORN"));
        assert!(output.contains("fade-delay = <5>"));
    }
}
