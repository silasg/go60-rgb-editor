#[cfg(test)]
mod integration_tests {
    use crate::parser::{parse_config, write_config};

    const SAMPLE_CONFIG: &str = include_str!("../../Go60 TK Latest RGB scheme.txt");

    #[test]
    fn test_parse_sample_config() {
        let config = parse_config(SAMPLE_CONFIG).expect("Failed to parse config");

        // Check that we have layers
        assert!(!config.layers.is_empty(), "Should have parsed layers");

        // Check that we have colors
        assert!(!config.palette.colors.is_empty(), "Should have parsed colors");

        // Check for specific known colors
        assert!(config.palette.get("RED").is_some(), "Should have RED color");
        assert!(config.palette.get("___").is_some(), "Should have ___ (off) color");
        assert!(config.palette.get("CYN").is_some(), "Should have CYN color");

        // Check for lock indicators
        assert!(config.palette.get("BSL").is_some(), "Should have BSL lock indicator");

        // Check for aliases
        assert!(config.palette.get("FST").is_some(), "Should have FST alias");
    }

    #[test]
    fn test_parse_layers() {
        let config = parse_config(SAMPLE_CONFIG).expect("Failed to parse config");

        // Check we have the expected number of layers
        assert!(config.layers.len() > 5, "Should have multiple layers");

        // Find the Cursor layer
        let cursor_layer = config
            .layers
            .iter()
            .find(|l| l.name == "Cursor")
            .expect("Should have Cursor layer");

        assert_eq!(cursor_layer.macro_name, "LAYER_Cursor");
        assert_eq!(cursor_layer.fade_delay, 5);
        assert_eq!(cursor_layer.left_half.len(), 6, "Should have 6 rows");
        assert_eq!(cursor_layer.right_half.len(), 6, "Should have 6 rows");
    }

    #[test]
    fn test_roundtrip() {
        let config = parse_config(SAMPLE_CONFIG).expect("Failed to parse config");
        let output = write_config(&config);

        // Parse the output again
        let reparsed = parse_config(&output).expect("Failed to reparse config");

        // Check layers match
        assert_eq!(
            config.layers.len(),
            reparsed.layers.len(),
            "Layer count should match"
        );

        for (orig, new) in config.layers.iter().zip(reparsed.layers.iter()) {
            assert_eq!(orig.name, new.name, "Layer names should match");
            assert_eq!(orig.macro_name, new.macro_name, "Macro names should match");
            assert_eq!(orig.fade_delay, new.fade_delay, "Fade delays should match");

            // Check all key positions
            for row in 0..orig.left_half.len() {
                for col in 0..orig.left_half[row].len() {
                    assert_eq!(
                        orig.left_half[row][col], new.left_half[row][col],
                        "Left half mismatch at ({}, {})",
                        row, col
                    );
                }
            }
            for row in 0..orig.right_half.len() {
                for col in 0..orig.right_half[row].len() {
                    assert_eq!(
                        orig.right_half[row][col], new.right_half[row][col],
                        "Right half mismatch at ({}, {})",
                        row, col
                    );
                }
            }
        }
    }

    #[test]
    fn test_special_colors() {
        let config = parse_config(SAMPLE_CONFIG).expect("Failed to parse config");

        // Check lock indicators are marked as special
        let bsl = config.palette.get("BSL").expect("Should have BSL");
        assert!(bsl.is_special(), "BSL should be marked as special");

        // Check aliases are marked as special
        let fst = config.palette.get("FST").expect("Should have FST");
        assert!(fst.is_special(), "FST should be marked as special");

        // Check regular colors are not special
        let red = config.palette.get("RED").expect("Should have RED");
        assert!(!red.is_special(), "RED should not be marked as special");
    }
}
