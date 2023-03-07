//! ## `binroots::fileserializer`
//! Contains [`SerializerError`][`crate::fileserializer::SerializerError`] and several internal definitions for serializing data into a file structure.

use serde::Serialize;

type SerializerResult<T> = std::result::Result<T, SerializerError>;

/// Errors during binroots' serialization process.
#[derive(Debug)]
pub enum SerializerError {
    /// A message from the serializer
    Message(String),
}

impl serde::ser::Error for SerializerError {
    fn custom<T: std::fmt::Display>(msg: T) -> Self {
        SerializerError::Message(msg.to_string())
    }
}

impl std::fmt::Display for SerializerError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            SerializerError::Message(msg) => formatter.write_str(msg),
        }
    }
}

impl std::error::Error for SerializerError {}

#[derive(Default, Debug, PartialEq)]
pub(crate) enum FileOperationHint {
    #[default]
    None,
    Delete,
    DeleteValue,
}

#[derive(Default, Debug, PartialEq)]
pub(crate) struct File {
    pub(crate) path: String,
    pub(crate) name: Option<String>,
    pub(crate) variant: Option<String>,
    pub(crate) output: Vec<u8>,
    pub(crate) is_path: bool,
    pub(crate) hint: FileOperationHint,
    pub(crate) folder_variant: Option<String>,
}

#[derive(Default, Debug, PartialEq)]
pub(crate) struct FileSerializer {
    name: Option<String>,
    pub(crate) root: String,
    file: usize,
    seq_level: usize,
    seq: Vec<usize>,
    is_key: bool,
    future_name: Option<String>,
    variant: Option<String>,
    pub(crate) output: Vec<File>,
    folder_variant: Option<String>,
}

impl FileSerializer {
    pub fn advance(&mut self) {
        self.name = None;
        self.file += 1;
    }

    pub fn inc_seq(&mut self, by: usize) -> usize {
        self.seq[self.seq_level - 1] += by;
        self.seq[self.seq_level - 1]
    }

    pub fn construct_seq(&mut self) {
        self.seq_level += 1;
    }

    pub fn destruct_seq(&mut self) {
        self.seq.pop().unwrap();
        self.seq_level -= 1;
    }

    pub fn seq_to_path(&self) -> String {
        let mut seqstr = self.seq.iter().map(|i| i.to_string()).collect::<Vec<_>>();
        seqstr.pop();
        seqstr.join("/")
    }

    pub fn write<A: Into<Vec<u8>> + std::fmt::Debug>(&mut self, data: A) {
        if self.is_key {
            self.future_name = Some(std::str::from_utf8(&data.into()).unwrap().into());
            self.file -= 1;
        } else {
            if self.seq_level > 0 {
                if self.variant.is_none() && self.seq.len() == self.seq_level {
                    self.inc_seq(1);
                } else {
                    self.seq.push(0);
                }
            }

            let name = if self.seq_level > 0 {
                Some((self.inc_seq(0)).to_string())
            } else {
                self.name.clone()
            };

            let path = {
                let seqpath = self.seq_to_path();
                if seqpath.is_empty() {
                    self.root.clone()
                } else if self.root.is_empty() {
                    seqpath
                } else {
                    format!("{}/{seqpath}", self.root.clone())
                }
            };

            let file = File {
                name,
                path,
                output: data.into(),
                variant: self.variant.clone(),
                is_path: false,
                hint: FileOperationHint::None,
                folder_variant: self.folder_variant.clone(),
            };

            self.output.push(file);
        }
    }

    pub fn prev(&mut self) -> &mut File {
        let len = self.output.len();
        &mut self.output[len - 1]
    }
}

impl<'a> serde::Serializer for &'a mut FileSerializer {
    type Ok = ();
    type Error = SerializerError;

    type SerializeSeq = Self;
    type SerializeTuple = Self;
    type SerializeTupleStruct = Self;
    type SerializeTupleVariant = Self;
    type SerializeMap = Self;
    type SerializeStruct = Self;
    type SerializeStructVariant = Self;

    fn serialize_bool(self, v: bool) -> SerializerResult<()> {
        self.advance();
        self.write(if v {
            b"true".to_vec()
        } else {
            b"false".to_vec()
        });
        Ok(())
    }

    fn serialize_i8(self, v: i8) -> SerializerResult<()> {
        self.serialize_i64(i64::from(v))
    }

    fn serialize_i16(self, v: i16) -> SerializerResult<()> {
        self.serialize_i64(i64::from(v))
    }

    fn serialize_i32(self, v: i32) -> SerializerResult<()> {
        self.serialize_i64(i64::from(v))
    }

    fn serialize_i64(self, v: i64) -> SerializerResult<()> {
        self.advance();
        self.write(itoa::Buffer::new().format(v).as_bytes());
        Ok(())
    }

    fn serialize_u8(self, v: u8) -> SerializerResult<()> {
        self.serialize_u64(u64::from(v))
    }

    fn serialize_u16(self, v: u16) -> SerializerResult<()> {
        self.serialize_u64(u64::from(v))
    }

    fn serialize_u32(self, v: u32) -> SerializerResult<()> {
        self.serialize_u64(u64::from(v))
    }

    fn serialize_u64(self, v: u64) -> SerializerResult<()> {
        self.advance();
        self.write(itoa::Buffer::new().format(v).as_bytes());
        Ok(())
    }

    fn serialize_f32(self, v: f32) -> SerializerResult<()> {
        self.serialize_f64(f64::from(v))
    }

    fn serialize_f64(self, v: f64) -> SerializerResult<()> {
        self.advance();
        self.write(ryu::Buffer::new().format(v).as_bytes());
        Ok(())
    }

    fn serialize_char(self, v: char) -> SerializerResult<()> {
        self.advance();
        let mut buf = [0; 4];
        self.write(v.encode_utf8(buf.as_mut_slice()).as_bytes());
        Ok(())
    }

    fn serialize_str(self, v: &str) -> SerializerResult<()> {
        self.advance();
        self.write(v.as_bytes());
        Ok(())
    }

    fn serialize_bytes(self, v: &[u8]) -> SerializerResult<()> {
        self.advance();
        self.write(v);
        Ok(())
    }

    fn serialize_none(self) -> SerializerResult<()> {
        self.advance();

        if self.is_key {
            self.write(b"__NONE__".to_vec());
        } else {
            self.write(Vec::new());
            self.prev().hint = FileOperationHint::Delete;
        }

        Ok(())
    }

    fn serialize_unit(self) -> SerializerResult<()> {
        self.advance();
        if self.is_key {
            self.write(b"__UNIT__".to_vec());
        } else {
            self.write(Vec::new());
        }
        Ok(())
    }

    fn serialize_some<T: ?Sized + serde::Serialize>(self, value: &T) -> SerializerResult<()> {
        value.serialize(self)
    }

    fn serialize_unit_struct(self, _name: &'static str) -> SerializerResult<()> {
        self.serialize_unit()
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> SerializerResult<()> {
        self.advance();
        self.write(variant.as_bytes());
        self.prev().hint = FileOperationHint::DeleteValue;
        Ok(())
    }

    fn serialize_newtype_struct<T: ?Sized + serde::Serialize>(
        self,
        _name: &'static str,
        value: &T,
    ) -> SerializerResult<()> {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T: ?Sized + serde::Serialize>(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> SerializerResult<()> {
        self.variant = Some("value".into());
        value.serialize(&mut *self)?;
        self.prev().hint = FileOperationHint::DeleteValue;
        self.variant = None;
        self.advance();
        self.write(variant.as_bytes());
        self.output[self.file - 1].name = self.output[self.file - 2].name.clone();

        Ok(())
    }

    fn serialize_seq(self, _len: Option<usize>) -> SerializerResult<Self::SerializeSeq> {
        self.advance();
        self.write(Vec::new());
        self.output[self.file - 1].is_path = true;
        self.construct_seq();

        Ok(self)
    }

    fn serialize_tuple(self, len: usize) -> SerializerResult<Self::SerializeTuple> {
        self.serialize_seq(Some(len))
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> SerializerResult<Self::SerializeTupleStruct> {
        self.serialize_seq(Some(len))
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        _len: usize,
    ) -> SerializerResult<Self::SerializeTupleVariant> {
        self.variant = Some("value".into());
        self.advance();
        self.write(Vec::new());
        self.prev().hint = FileOperationHint::DeleteValue;
        self.output[self.file - 1].is_path = true;
        self.variant = None;

        self.advance();
        self.write(variant.as_bytes());
        self.construct_seq();

        Ok(self)
    }

    fn serialize_map(self, _len: Option<usize>) -> SerializerResult<Self::SerializeMap> {
        self.advance();
        self.write(Vec::new());
        self.output[self.file - 1].is_path = true;
        Ok(self)
    }

    fn serialize_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> SerializerResult<Self::SerializeStruct> {
        self.serialize_map(Some(len))
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        _len: usize,
    ) -> SerializerResult<Self::SerializeStructVariant> {
        self.variant = Some("value".into());
        self.advance();
        self.write(Vec::new());
        self.prev().hint = FileOperationHint::DeleteValue;
        self.output[self.file - 1].is_path = true;
        self.variant = None;

        self.advance();
        self.write(variant.as_bytes());

        Ok(self)
    }
}

impl<'a> serde::ser::SerializeSeq for &'a mut FileSerializer {
    type Ok = ();
    type Error = SerializerError;

    fn serialize_element<T>(&mut self, value: &T) -> SerializerResult<()>
    where
        T: ?Sized + serde::Serialize,
    {
        value.serialize(&mut **self)?;

        Ok(())
    }

    fn end(self) -> SerializerResult<()> {
        self.destruct_seq();
        Ok(())
    }
}

impl<'a> serde::ser::SerializeTuple for &'a mut FileSerializer {
    type Ok = ();
    type Error = SerializerError;

    fn serialize_element<T>(&mut self, value: &T) -> SerializerResult<()>
    where
        T: ?Sized + serde::Serialize,
    {
        value.serialize(&mut **self)?;

        Ok(())
    }

    fn end(self) -> SerializerResult<()> {
        self.destruct_seq();
        Ok(())
    }
}

impl<'a> serde::ser::SerializeTupleStruct for &'a mut FileSerializer {
    type Ok = ();
    type Error = SerializerError;

    fn serialize_field<T>(&mut self, value: &T) -> SerializerResult<()>
    where
        T: ?Sized + serde::Serialize,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> SerializerResult<()> {
        self.destruct_seq();
        Ok(())
    }
}

impl<'a> serde::ser::SerializeTupleVariant for &'a mut FileSerializer {
    type Ok = ();
    type Error = SerializerError;

    fn serialize_field<T>(&mut self, value: &T) -> SerializerResult<()>
    where
        T: ?Sized + serde::Serialize,
    {
        self.folder_variant = Some("value".into());
        value.serialize(&mut **self)?;
        self.folder_variant = None;

        Ok(())
    }

    fn end(self) -> SerializerResult<()> {
        self.destruct_seq();
        Ok(())
    }
}

impl<'a> serde::ser::SerializeMap for &'a mut FileSerializer {
    type Ok = ();
    type Error = SerializerError;

    fn serialize_key<T>(&mut self, key: &T) -> SerializerResult<()>
    where
        T: ?Sized + serde::Serialize,
    {
        #[cfg(debug_assertions)]
        {
            let valid_keys = [
                "String", "&str", "str", "u8", "u16", "u32", "u64", "i8", "i16", "i32", "i64",
                "f32", "f64", "()",
            ];
            let type_name = &std::any::type_name::<T>()
                .trim_start_matches("&")
                .trim_start_matches("std::")
                .trim_start_matches("core::");

            let is_valid_key = valid_keys.iter().fold(false, |acc, x| {
                acc || (type_name == x) || (type_name == &format!("option::Option<{x}>"))
            });

            if !is_valid_key {
                panic!(
                    "Can't serialize a HashMap with the key of {}. Must be one of {valid_keys:?} or an Option containing one of them.",
                    type_name,
                )
            }
        }

        self.is_key = true;
        key.serialize(&mut **self)
    }

    fn serialize_value<T>(&mut self, value: &T) -> SerializerResult<()>
    where
        T: ?Sized + serde::Serialize,
    {
        self.is_key = false;
        if let Some(future_name) = &self.future_name {
            self.root += &format!("/{}", future_name);
        }
        value.serialize(&mut **self)?;
        let split = self.root.split("/");
        self.root = split
            .clone()
            .take(split.count() - 1)
            .collect::<Vec<_>>()
            .join("/");

        Ok(())
    }

    fn end(self) -> SerializerResult<()> {
        Ok(())
    }
}

impl<'a> serde::ser::SerializeStruct for &'a mut FileSerializer {
    type Ok = ();
    type Error = SerializerError;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> SerializerResult<()>
    where
        T: ?Sized + serde::Serialize,
    {
        self.is_key = true;
        key.serialize(&mut **self)?;
        self.is_key = false;
        if let Some(future_name) = &self.future_name {
            self.root += &format!("/{}", future_name);
        }
        value.serialize(&mut **self)?;
        let split = self.root.split("/");
        self.root = split
            .clone()
            .take(split.count() - 1)
            .collect::<Vec<_>>()
            .join("/");

        Ok(())
    }

    fn end(self) -> SerializerResult<()> {
        Ok(())
    }
}

impl<'a> serde::ser::SerializeStructVariant for &'a mut FileSerializer {
    type Ok = ();
    type Error = SerializerError;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> SerializerResult<()>
    where
        T: ?Sized + serde::Serialize,
    {
        self.is_key = true;

        key.serialize(&mut **self)?;

        self.is_key = false;
        if let Some(future_name) = &self.future_name {
            self.root += &format!("/{}", future_name);
        }
        self.folder_variant = Some("value".into());
        value.serialize(&mut **self)?;
        let split = self.root.split("/");
        self.root = split
            .clone()
            .take(split.count() - 1)
            .collect::<Vec<_>>()
            .join("/");
        self.folder_variant = None;

        Ok(())
    }

    fn end(self) -> SerializerResult<()> {
        Ok(())
    }
}
