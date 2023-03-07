//! # `binroots`
//! Binroots is a (cross-platform!) crate that provides a simple and efficient way to save Rust data structures to disk. It allows you to save each field of a struct or enum variant as a separate file, making it easy to store reactive data, allowing end-users and hackers to watch individual files for changes and automate command-line tools for your app.
//!
//! ## Setting up a struct - see [`binroots_struct`][`crate::binroots_struct`]
//! ```rust
//! use binroots::binroots_struct;
//!
//! # #[binroots::binroots_enum]
//! # enum Activity {
//! #     None, // <- Automatically chosen as the default value
//! #     Playing(String),
//! # }
//!
//! #[binroots_struct]
//! struct Status {
//!     connections: usize,
//!     is_online: bool,
//!     activity: Activity,
//! }
//! ```
//!
//! ## Setting up an enum - see [`binroots_enum`][`crate::binroots_enum`]
//! ```rust
//! use binroots::binroots_enum;
//!
//! #[binroots_enum]
//! enum Activity {
//!     None, // <- Automatically chosen as the default value
//!     Playing(String),
//! }
//! ```
//!
//! ## Saving data - see [`Save::save`][`crate::save::Save::save`] and [`binroots_struct`][`crate::binroots_struct`]
//! ```rust
//! # use binroots::{binroots_enum, binroots_struct};
//! #[binroots_enum]
//! # enum Activity {
//! #     None, // <- Automatically chosen as the default value
//! #     Playing(String),
//! # }
//! # #[binroots_struct]
//! # struct Status {
//! #     connections: usize,
//! #     is_online: bool,
//! #     activity: Activity,
//! # }
//!
//! use binroots::save::{RootType, SaveError};
//!
//! fn main() -> Result<(), SaveError> {
//!     let mut status = Status::default();
//!
//!     *status.is_online = true;
//!     status.save()?; // <- Saves the entire struct to the disk
//!
//!     *status.activity = Activity::Playing("video gamb".into());
//!     status.activity.save(Status::ROOT_FOLDER, RootType::InMemory)?; // <- Only saves status.activity to the disk
//!
//!     Ok(())
//! }
//! ```

#![feature(adt_const_params)]
#![feature(io_error_more)]
#![warn(missing_docs)]

pub mod field;
pub mod fileserializer;
pub mod save;

pub use binroots_proc_macros::*;
pub use serde::Serialize;

#[cfg(test)]
mod tests {
    macro_rules! assert_file {
        ($path: expr, $contents: expr) => {
            assert_eq!(
                std::fs::read_to_string(
                    crate::save::root_location(crate::save::RootType::InMemory)
                        .unwrap()
                        .join($path)
                )
                .unwrap(),
                $contents
            )
        };
    }

    use std::collections::HashMap;

    use crate::save::{root_location, RootType::*, Save};
    use crate::*;

    #[test]
    fn serialize_field() {
        let field = field::BinrootsField::<"fieldname", &str>::new("Hello");

        let mut serializer1 = fileserializer::FileSerializer::default();
        let mut serializer2 = fileserializer::FileSerializer::default();

        field.serialize(&mut serializer1).unwrap();
        (*field).serialize(&mut serializer2).unwrap();

        assert_eq!(serializer1, serializer2);
    }

    #[test]
    fn save_bool() {
        (true, false).save("test_save_bool", InMemory).unwrap();
        assert_file!("test_save_bool/0", "true");
        assert_file!("test_save_bool/1", "false");
    }

    #[test]
    fn save_int() {
        (-128, 777).save("test_save_int", InMemory).unwrap();
        assert_file!("test_save_int/0", "-128");
        assert_file!("test_save_int/1", "777");
    }

    #[test]
    fn save_float() {
        (-0.0, 0.0, 2.0 + 1.0)
            .save("test_save_float", InMemory)
            .unwrap();
        assert_file!("test_save_float/0", "-0.0");
        assert_file!("test_save_float/1", "0.0");
        assert_file!("test_save_float/2", "3.0");
    }

    #[test]
    fn save_char() {
        ('🥺', 'è', 'e').save("test_save_char", InMemory).unwrap();
        assert_file!("test_save_char/0", "🥺");
        assert_file!("test_save_char/1", "è");
        assert_file!("test_save_char/2", "e");
    }

    #[test]
    fn save_str() {
        (
            "👨‍👩‍👦‍👦".to_string(),
            "Hello, world!".to_string(),
            "ﾟ･✿ヾ╲(｡◕‿◕｡)╱✿･ﾟ".to_string(),
        )
            .save("test_save_str", InMemory)
            .unwrap();
        assert_file!("test_save_str/0", "👨‍👩‍👦‍👦");
        assert_file!("test_save_str/1", "Hello, world!");
        assert_file!("test_save_str/2", "ﾟ･✿ヾ╲(｡◕‿◕｡)╱✿･ﾟ");
    }

    #[test]
    fn save_bytes() {
        #[derive(Serialize)]
        struct Efficient<'a> {
            #[serde(with = "serde_bytes")]
            bytes: &'a [u8],
        }

        Efficient {
            bytes: (&[
                240u8, 159, 145, 168, 226, 128, 141, 240, 159, 145, 169, 226, 128, 141, 240, 159,
                145, 166, 226, 128, 141, 240, 159, 145, 166,
            ]),
        }
        .save("test_save_bytes", InMemory)
        .unwrap();

        assert_file!("test_save_bytes/bytes", "👨‍👩‍👦‍👦");
    }

    #[test]
    fn save_empty() {
        ().save("test_save_empty.unit", InMemory).unwrap();
        assert_file!("test_save_empty.unit", "");

        #[derive(Serialize)]
        struct Nothing;

        Nothing
            .save("test_save_empty.unit-struct", InMemory)
            .unwrap();
        assert_file!("test_save_empty.unit-struct", "");
    }

    #[test]
    fn save_none_will_delete() {
        Some("Hello").save("test_save_none", InMemory).unwrap();
        assert_file!("test_save_none", "Hello");

        None::<()>.save("test_save_none", InMemory).unwrap();
        assert_eq!(
            std::fs::File::open(root_location(InMemory).unwrap().join("test_save_none"))
                .unwrap_err()
                .kind(),
            std::io::ErrorKind::NotFound
        )
    }

    #[test]
    fn save_unit_variant() {
        #[derive(Serialize)]
        enum E {
            Bin,
        }

        E::Bin.save("test_save_unit_variant", InMemory).unwrap();
        assert_file!("test_save_unit_variant", "Bin");
    }

    #[test]
    fn save_newtype_struct() {
        #[derive(Serialize)]
        struct Plain(String);

        Plain("Jane".into())
            .save("test_save_newtype_struct", InMemory)
            .unwrap();
        assert_file!("test_save_newtype_struct", "Jane");
    }

    #[test]
    fn save_newtype_variant() {
        #[derive(Serialize)]
        enum E {
            Bin(String),
        }

        E::Bin("Roots".into())
            .save("test_save_newtype_variant", InMemory)
            .unwrap();
        assert_file!("test_save_newtype_variant", "Bin");
        assert_file!("test_save_newtype_variant.value", "Roots");
    }

    #[test]
    fn save_seq() {
        [9, 8, 5].save("test_save_seq", InMemory).unwrap();
        assert_file!("test_save_seq/0", "9");
        assert_file!("test_save_seq/1", "8");
        assert_file!("test_save_seq/2", "5");
    }

    #[test]
    fn save_seq_multiple() {
        [[9, 8, 5], [4, 1, 1]]
            .save("test_save_seq_multiple", InMemory)
            .unwrap();

        assert_file!("test_save_seq_multiple/0/0", "9");
        assert_file!("test_save_seq_multiple/0/1", "8");
        assert_file!("test_save_seq_multiple/0/2", "5");
        assert_file!("test_save_seq_multiple/1/0", "4");
        assert_file!("test_save_seq_multiple/1/1", "1");
        assert_file!("test_save_seq_multiple/1/2", "1");
    }

    #[test]
    fn save_tuple() {
        (9, 8, 5).save("test_save_tuple", InMemory).unwrap();
        assert_file!("test_save_tuple/0", "9");
        assert_file!("test_save_tuple/1", "8");
        assert_file!("test_save_tuple/2", "5");
    }

    #[test]
    fn save_tuple_multiple() {
        ((9, 8, 5), (4, 1, 1))
            .save("test_save_tuple_multiple", InMemory)
            .unwrap();

        assert_file!("test_save_tuple_multiple/0/0", "9");
        assert_file!("test_save_tuple_multiple/0/1", "8");
        assert_file!("test_save_tuple_multiple/0/2", "5");
        assert_file!("test_save_tuple_multiple/1/0", "4");
        assert_file!("test_save_tuple_multiple/1/1", "1");
        assert_file!("test_save_tuple_multiple/1/2", "1");
    }

    #[test]
    fn save_tuple_struct() {
        #[derive(Serialize)]
        struct Germy(u8, u8, u8);
        Germy(9, 8, 5)
            .save("test_save_tuple_struct", InMemory)
            .unwrap();
        assert_file!("test_save_tuple_struct/0", "9");
        assert_file!("test_save_tuple_struct/1", "8");
        assert_file!("test_save_tuple_struct/2", "5");
    }

    #[test]
    fn save_tuple_variant() {
        #[derive(Serialize)]
        enum E {
            Germy(u8, u8, u8),
        }
        E::Germy(9, 8, 5)
            .save("test_save_tuple_variant", InMemory)
            .unwrap();
        assert_file!("test_save_tuple_variant", "Germy");
        assert_file!("test_save_tuple_variant.value/0", "9");
        assert_file!("test_save_tuple_variant.value/1", "8");
        assert_file!("test_save_tuple_variant.value/2", "5");
    }

    #[test]
    fn save_map() {
        let mut map = HashMap::new();
        map.insert("bin", "roots");
        map.insert("roots", "very cool");

        map.save("test_save_map", InMemory).unwrap();
        assert_file!("test_save_map/bin", "roots");
        assert_file!("test_save_map/roots", "very cool");
    }

    #[test]
    fn save_map_option() {
        let mut map = HashMap::new();
        map.insert(Some("bin"), "roots");
        map.insert(None, "very cool");

        map.save("test_save_map_option", InMemory).unwrap();
        assert_file!("test_save_map_option/bin", "roots");
        assert_file!("test_save_map_option/__NONE__", "very cool");
    }

    #[test]
    fn save_map_unit() {
        let mut map = HashMap::new();
        map.insert((), "roots");

        map.save("test_save_map_unit", InMemory).unwrap();
        assert_file!("test_save_map_unit/__UNIT__", "roots");
    }

    #[test]
    fn save_map_array_value() {
        let mut map: HashMap<_, &[i32]> = HashMap::new();
        map.insert("germy", &[9, 8, 5]);
        map.insert("year", &[2, 0, 2, 3]);

        map.save("test_save_map_array_value", InMemory).unwrap();
        assert_file!("test_save_map_array_value/germy/0", "9");
        assert_file!("test_save_map_array_value/germy/1", "8");
        assert_file!("test_save_map_array_value/germy/2", "5");
        assert_file!("test_save_map_array_value/year/0", "2");
        assert_file!("test_save_map_array_value/year/1", "0");
        assert_file!("test_save_map_array_value/year/2", "2");
        assert_file!("test_save_map_array_value/year/3", "3");
    }

    #[test]
    fn save_struct() {
        #[derive(Serialize)]
        struct Rgb {
            r: u8,
            g: u8,
            b: u8,
        }

        Rgb {
            r: 40,
            g: 60,
            b: 80,
        }
        .save("test_save_struct", InMemory)
        .unwrap();
        assert_file!("test_save_struct/r", "40");
        assert_file!("test_save_struct/g", "60");
        assert_file!("test_save_struct/b", "80");
    }

    #[test]
    fn save_struct_variant() {
        #[derive(Serialize)]
        enum E {
            Rgb { r: u8, g: u8, b: u8 },
        }

        E::Rgb {
            r: 40,
            g: 60,
            b: 80,
        }
        .save("test_save_struct_variant", InMemory)
        .unwrap();
        assert_file!("test_save_struct_variant", "Rgb");
        assert_file!("test_save_struct_variant.value/r", "40");
        assert_file!("test_save_struct_variant.value/g", "60");
        assert_file!("test_save_struct_variant.value/b", "80");
    }

    #[test]
    fn save_enums_will_delete_values() {
        #[derive(Serialize)]
        enum E {
            Unit,
            Single(u8),
            Tuple(u8, u8),
            Threeple { r: u8, g: u8 },
        }

        E::Threeple { r: 34, g: 40 }
            .save("test_save_enums_will_delete_values", InMemory)
            .unwrap();

        assert_file!("test_save_enums_will_delete_values", "Threeple");
        assert_file!("test_save_enums_will_delete_values.value/r", "34");
        assert_file!("test_save_enums_will_delete_values.value/g", "40");

        E::Single(12)
            .save("test_save_enums_will_delete_values", InMemory)
            .unwrap();

        assert_file!("test_save_enums_will_delete_values", "Single");
        assert_file!("test_save_enums_will_delete_values.value", "12");

        E::Tuple(8, 4)
            .save("test_save_enums_will_delete_values", InMemory)
            .unwrap();

        assert_file!("test_save_enums_will_delete_values", "Tuple");
        assert_file!("test_save_enums_will_delete_values.value/0", "8");
        assert_file!("test_save_enums_will_delete_values.value/1", "4");

        E::Unit
            .save("test_save_enums_will_delete_values", InMemory)
            .unwrap();

        assert_file!("test_save_enums_will_delete_values", "Unit");
    }
}
