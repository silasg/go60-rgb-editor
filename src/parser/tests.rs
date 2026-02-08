#[cfg(test)]
mod integration_tests {
    use crate::model::{ColorDef, ColorKind, ColorPalette, Config, Layer, RgbColor};
    use crate::parser::{parse_config, write_config};
    use std::path::PathBuf;

    const SAMPLE_CONFIG: &str = include_str!("../../Go60 TK Latest RGB scheme.txt");

    #[test]
    fn test_parse_sample_config() {
        // Act
        let config = parse_config(SAMPLE_CONFIG).expect("Failed to parse config");

        // Assert
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
        // Act
        let config = parse_config(SAMPLE_CONFIG).expect("Failed to parse config");
        let cursor_layer = config
            .layers
            .iter()
            .find(|l| l.name == "Cursor")
            .expect("Should have Cursor layer");

        // Assert
        assert!(config.layers.len() > 5, "Should have multiple layers");
        assert_eq!(cursor_layer.macro_name, "LAYER_Cursor");
        assert_eq!(cursor_layer.fade_delay, 5);
        assert_eq!(cursor_layer.left_half.len(), 6, "Should have 6 rows");
        assert_eq!(cursor_layer.right_half.len(), 6, "Should have 6 rows");
    }

    #[test]
    fn test_roundtrip() {
        // Act
        let config = parse_config(SAMPLE_CONFIG).expect("Failed to parse config");
        let output = write_config(&config);
        let reparsed = parse_config(&output).expect("Failed to reparse config");

        // Assert
        assert_eq!(
            config.layers.len(),
            reparsed.layers.len(),
            "Layer count should match"
        );
        for (orig, new) in config.layers.iter().zip(reparsed.layers.iter()) {
            assert_eq!(orig.name, new.name, "Layer names should match");
            assert_eq!(orig.macro_name, new.macro_name, "Macro names should match");
            assert_eq!(orig.fade_delay, new.fade_delay, "Fade delays should match");
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
        // Act
        let config = parse_config(SAMPLE_CONFIG).expect("Failed to parse config");
        let lock_indicator_bsl = config.palette.get("BSL").expect("Should have BSL");
        let alias_fst = config.palette.get("FST").expect("Should have FST");
        let regular_color_red = config.palette.get("RED").expect("Should have RED");

        // Assert
        assert!(lock_indicator_bsl.is_special(), "BSL should be marked as special");
        assert!(alias_fst.is_special(), "FST should be marked as special");
        assert!(!regular_color_red.is_special(), "RED should not be marked as special");
    }

    #[test]
    fn test_serialize_complete_config_with_layers_to_zmk_format() {
        // Arrange
        let mut config = Config::new(PathBuf::from("test.txt"));
        config.raw_header = "// Test header\n".to_string();
        config.raw_footer = "// Test footer\n".to_string();
        let mut palette = ColorPalette::new();
        for (abbrev, hex) in [
            ("___", "000000"),
            ("BCL", "FF0000"),
            ("BNL", "FF0000"),
            ("BSL", "FF0000"),
            ("MAJ", "FF00FF"),
            ("CYN", "00FFFF"),
            ("CHU", "80FF00"),
            ("YLW", "FFFF00"),
            ("ORN", "FF8000"),
            ("PNK", "FF66B2"),
            ("WHT", "FFFFFF"),
            ("GRN", "00FF00"),
            ("DUG", "808080"),
        ] {
            palette.colors.push(ColorDef {
                abbrev: abbrev.to_string(),
                rgb_name: format!("{}_RGB", abbrev),
                rgb: RgbColor::from_hex(hex).unwrap(),
                kind: ColorKind::Regular,
                comment: None,
            });
            palette.by_abbrev.insert(abbrev.to_string(), palette.colors.len() - 1);
        }
        config.palette = palette;

        // Create HRM_WinLinx layer
        let mut hrm = Layer::new("HRM_WinLinx".to_string(), "LAYER_HRM_WinLinx".to_string());
        hrm.fade_delay = 30;
        hrm.left_half = vec![
            vec!["___", "___", "BCL", "BNL", "BSL", "___"].iter().map(|s| s.to_string()).collect(),
            vec!["___", "___", "___", "___", "___", "___"].iter().map(|s| s.to_string()).collect(),
            vec!["___", "MAJ", "CYN", "CHU", "YLW", "___"].iter().map(|s| s.to_string()).collect(),
            vec!["___", "___", "___", "___", "___", "___"].iter().map(|s| s.to_string()).collect(),
            vec!["___", "___", "___"].iter().map(|s| s.to_string()).collect(),
            vec!["___", "___", "___"].iter().map(|s| s.to_string()).collect(),
        ];
        hrm.right_half = vec![
            vec!["___", "___", "___", "___", "___", "___"].iter().map(|s| s.to_string()).collect(),
            vec!["___", "___", "___", "___", "___", "___"].iter().map(|s| s.to_string()).collect(),
            vec!["___", "YLW", "CHU", "CYN", "MAJ", "___"].iter().map(|s| s.to_string()).collect(),
            vec!["___", "___", "___", "___", "___", "___"].iter().map(|s| s.to_string()).collect(),
            vec!["___", "___", "___"].iter().map(|s| s.to_string()).collect(),
            vec!["___", "___", "___"].iter().map(|s| s.to_string()).collect(),
        ];
        config.layers.push(hrm);

        // Create Cursor layer
        let mut cursor = Layer::new("Cursor".to_string(), "LAYER_Cursor".to_string());
        cursor.fade_delay = 5;
        cursor.left_half = vec![
            vec!["___", "ORN", "ORN", "ORN", "ORN", "___"].iter().map(|s| s.to_string()).collect(),
            vec!["___", "ORN", "MAJ", "MAJ", "ORN", "YLW"].iter().map(|s| s.to_string()).collect(),
            vec!["WHT", "WHT", "WHT", "WHT", "WHT", "YLW"].iter().map(|s| s.to_string()).collect(),
            vec!["MAJ", "CYN", "CYN", "CYN", "MAJ", "YLW"].iter().map(|s| s.to_string()).collect(),
            vec!["___", "MAJ", "MAJ"].iter().map(|s| s.to_string()).collect(),
            vec!["___", "DUG", "___"].iter().map(|s| s.to_string()).collect(),
        ];
        cursor.right_half = vec![
            vec!["___", "___", "___", "___", "___", "___"].iter().map(|s| s.to_string()).collect(),
            vec!["YLW", "ORN", "PNK", "PNK", "ORN", "___"].iter().map(|s| s.to_string()).collect(),
            vec!["YLW", "GRN", "GRN", "GRN", "GRN", "WHT"].iter().map(|s| s.to_string()).collect(),
            vec!["YLW", "GRN", "GRN", "GRN", "GRN", "MAJ"].iter().map(|s| s.to_string()).collect(),
            vec!["MAJ", "MAJ", "MAJ"].iter().map(|s| s.to_string()).collect(),
            vec!["CYN", "CYN", "CYN"].iter().map(|s| s.to_string()).collect(),
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

        // Act
        let output = write_config(&config);

        // Assert
        assert_eq!(output, expected, "Serialized output doesn't match expected");
    }
}
