#[cfg(test)]
mod integration_tests {
    use crate::domain::{ColorDef, ColorKind, ColorPalette, Config, Layer, RgbColor};
    use crate::domain::parser::{parse_config, write_config};
    use std::path::PathBuf;

    const SAMPLE_CONFIG: &str = include_str!("../../../tests/fixtures/sample_config.txt");

    fn row(keys: &[&str]) -> Vec<String> {
        keys.iter().map(|s| s.to_string()).collect()
    }

    fn build_test_palette() -> ColorPalette {
        let mut palette = ColorPalette::new();
        for (abbrev, hex) in [
            ("___", "000000"), ("BCL", "FF0000"), ("BNL", "FF0000"),
            ("BSL", "FF0000"), ("MAJ", "FF00FF"), ("CYN", "00FFFF"),
            ("CHU", "80FF00"), ("YLW", "FFFF00"), ("ORN", "FF8000"),
            ("PNK", "FF66B2"), ("WHT", "FFFFFF"), ("GRN", "00FF00"),
            ("DUG", "808080"),
        ] {
            palette.add(ColorDef::new(abbrev.to_string(), RgbColor::from_hex(hex).unwrap()));
        }
        palette
    }

    #[test]
    fn test_parse_sample_config() {
        let config = parse_config(SAMPLE_CONFIG).expect("Failed to parse config");

        assert!(!config.layers.is_empty(), "Should have parsed layers");
        assert!(!config.palette.colors.is_empty(), "Should have parsed colors");
        assert!(config.palette.get("RED").is_some(), "Should have RED color");
        assert!(config.palette.get("___").is_some(), "Should have ___ (off) color");
        assert!(config.palette.get("CYN").is_some(), "Should have CYN color");
        assert!(config.palette.get("BSL").is_some(), "Should have BSL lock indicator");
        assert!(config.palette.get("FST").is_some(), "Should have FST alias");
    }

    #[test]
    fn test_parse_layers() {
        let config = parse_config(SAMPLE_CONFIG).expect("Failed to parse config");
        let cursor_layer = config
            .layers
            .iter()
            .find(|l| l.name == "Cursor")
            .expect("Should have Cursor layer");

        assert!(config.layers.len() > 5, "Should have multiple layers");
        assert_eq!(cursor_layer.macro_name, "LAYER_Cursor");
        assert_eq!(cursor_layer.fade_delay, 5);
        assert_eq!(cursor_layer.left_half.len(), 6, "Should have 6 rows");
        assert_eq!(cursor_layer.right_half.len(), 6, "Should have 6 rows");
    }

    #[test]
    fn test_roundtrip() {
        let config = parse_config(SAMPLE_CONFIG).expect("Failed to parse config");
        let output = write_config(&config);
        let reparsed = parse_config(&output).expect("Failed to reparse config");

        assert_eq!(config.layers.len(), reparsed.layers.len(), "Layer count should match");
        for (orig, new) in config.layers.iter().zip(reparsed.layers.iter()) {
            assert_eq!(orig.name, new.name, "Layer names should match");
            assert_eq!(orig.macro_name, new.macro_name, "Macro names should match");
            assert_eq!(orig.fade_delay, new.fade_delay, "Fade delays should match");
            for row in 0..orig.left_half.len() {
                for col in 0..orig.left_half[row].len() {
                    assert_eq!(
                        orig.left_half[row][col], new.left_half[row][col],
                        "Left half mismatch at ({}, {})", row, col
                    );
                }
            }
            for row in 0..orig.right_half.len() {
                for col in 0..orig.right_half[row].len() {
                    assert_eq!(
                        orig.right_half[row][col], new.right_half[row][col],
                        "Right half mismatch at ({}, {})", row, col
                    );
                }
            }
        }
    }

    #[test]
    fn test_special_colors() {
        let config = parse_config(SAMPLE_CONFIG).expect("Failed to parse config");
        let lock_indicator_bsl = config.palette.get("BSL").expect("Should have BSL");
        let alias_fst = config.palette.get("FST").expect("Should have FST");
        let regular_color_red = config.palette.get("RED").expect("Should have RED");

        assert!(matches!(lock_indicator_bsl.kind, ColorKind::LockIndicator { .. }), "BSL should be a lock indicator");
        assert!(matches!(alias_fst.kind, ColorKind::Alias { .. }), "FST should be an alias");
        assert!(matches!(regular_color_red.kind, ColorKind::Regular), "RED should be a regular color");
    }

    #[test]
    fn test_serialize_complete_config_with_layers_to_zmk_format() {
        let mut config = Config::new(PathBuf::from("test.txt"));
        config.raw_header = "// Test header\n".to_string();
        config.raw_footer = "// Test footer\n".to_string();
        config.palette = build_test_palette();

        let mut hrm = Layer::new("HRM_WinLinx".to_string(), "LAYER_HRM_WinLinx".to_string());
        hrm.fade_delay = 30;
        hrm.left_half = vec![
            row(&["___", "___", "BCL", "BNL", "BSL", "___"]),
            row(&["___", "___", "___", "___", "___", "___"]),
            row(&["___", "MAJ", "CYN", "CHU", "YLW", "___"]),
            row(&["___", "___", "___", "___", "___", "___"]),
            row(&["___", "___", "___"]),
            row(&["___", "___", "___"]),
        ];
        hrm.right_half = vec![
            row(&["___", "___", "___", "___", "___", "___"]),
            row(&["___", "___", "___", "___", "___", "___"]),
            row(&["___", "YLW", "CHU", "CYN", "MAJ", "___"]),
            row(&["___", "___", "___", "___", "___", "___"]),
            row(&["___", "___", "___"]),
            row(&["___", "___", "___"]),
        ];
        config.layers.push(hrm);

        let mut cursor = Layer::new("Cursor".to_string(), "LAYER_Cursor".to_string());
        cursor.fade_delay = 5;
        cursor.left_half = vec![
            row(&["___", "ORN", "ORN", "ORN", "ORN", "___"]),
            row(&["___", "ORN", "MAJ", "MAJ", "ORN", "YLW"]),
            row(&["WHT", "WHT", "WHT", "WHT", "WHT", "YLW"]),
            row(&["MAJ", "CYN", "CYN", "CYN", "MAJ", "YLW"]),
            row(&["___", "MAJ", "MAJ"]),
            row(&["___", "DUG", "___"]),
        ];
        cursor.right_half = vec![
            row(&["___", "___", "___", "___", "___", "___"]),
            row(&["YLW", "ORN", "PNK", "PNK", "ORN", "___"]),
            row(&["YLW", "GRN", "GRN", "GRN", "GRN", "WHT"]),
            row(&["YLW", "GRN", "GRN", "GRN", "GRN", "MAJ"]),
            row(&["MAJ", "MAJ", "MAJ"]),
            row(&["CYN", "CYN", "CYN"]),
        ];
        config.layers.push(cursor);

        let expected = r#"// Test header
// ==== PER-KEY-RGB <section begins> ====
  / {
    underglow-layer {
      compatible = "zmk,underglow-layer";

      #ifdef LAYER_HRM_WinLinx
      HRM_WinLinx {
        bindings = <
          ___ ___ BCL BNL BSL ___                                     ___ ___ ___ ___ ___ ___
          ___ ___ ___ ___ ___ ___                                     ___ ___ ___ ___ ___ ___
          ___ MAJ CYN CHU YLW ___                                     ___ YLW CHU CYN MAJ ___
          ___ ___ ___ ___ ___ ___                                     ___ ___ ___ ___ ___ ___
                  ___ ___ ___                                             ___ ___ ___ 
                                      ___ ___ ___     ___ ___ ___ 
        >;
        layer-id = <LAYER_HRM_WinLinx>;
        fade-delay = <30>;
      };
      #endif

      #ifdef LAYER_Cursor
      Cursor {
        bindings = <
          ___ ORN ORN ORN ORN ___                                     ___ ___ ___ ___ ___ ___
          ___ ORN MAJ MAJ ORN YLW                                     YLW ORN PNK PNK ORN ___
          WHT WHT WHT WHT WHT YLW                                     YLW GRN GRN GRN GRN WHT
          MAJ CYN CYN CYN MAJ YLW                                     YLW GRN GRN GRN GRN MAJ
                  ___ MAJ MAJ                                             MAJ MAJ MAJ 
                                      ___ DUG ___     CYN CYN CYN 
        >;
        layer-id = <LAYER_Cursor>;
        fade-delay = <5>;
      };
      #endif

    };
  };
  // ==== PER-KEY-RGB <section ends> =====
// Test footer
"#;

        let output = write_config(&config);

        assert_eq!(output, expected, "Serialized output doesn't match expected");
    }
}
