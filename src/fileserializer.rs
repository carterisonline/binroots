//! ## `binroots::fileserializer`
//! Contains [`SerializerError`][`crate::fileserializer::SerializerError`] and several internal definitions for serializing data into a file structure.

use serde::Serialize;
use tracing::instrument;

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

#[derive(Default, Debug)]
pub(crate) struct File {
    pub(crate) path: String,
    pub(crate) name: Option<String>,
    pub(crate) variant: Option<String>,
    pub(crate) output: Vec<u8>,
    pub(crate) is_path: bool,
}

#[derive(Default, Debug)]
pub(crate) struct FileSerializer {
    name: Option<String>,
    pub(crate) root: String,
    file: usize,
    in_seq: bool,
    seq_n: usize,
    is_key: bool,
    future_name: Option<String>,
    variant: Option<String>,
    pub(crate) output: Vec<File>,
}

impl FileSerializer {
    #[instrument]
    pub fn advance(&mut self) {
        self.name = None;
        self.file += 1;
    }

    #[instrument]
    pub fn advance_into<S: AsRef<str> + std::fmt::Debug>(&mut self, name: S) {
        self.advance();
        self.name = Some(name.as_ref().into());
        self.root = format!("{}/{}", self.root, name.as_ref());
    }

    #[instrument]
    pub fn write<A: Into<Vec<u8>> + std::fmt::Debug>(&mut self, data: A) {
        //assert!(self.file == self.output.len());

        if self.is_key {
            self.future_name = Some(std::str::from_utf8(&data.into()).unwrap().into());
            self.file -= 1;
        } else {
            self.output.push(File {
                path: self.root.clone(),
                name: if let Some(name) = self.future_name.clone() {
                    self.future_name = None;
                    Some(name)
                } else if self.in_seq {
                    if self.variant.is_none() {
                        self.seq_n += 1;
                    }
                    Some((self.seq_n - if self.variant.is_none() { 1 } else { 0 }).to_string())
                } else {
                    self.name.clone()
                },
                output: data.into(),
                variant: self.variant.clone(),
                is_path: false,
            });
        }
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
        let mut buf = Vec::new();
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
        self.serialize_unit()
    }

    fn serialize_unit(self) -> SerializerResult<()> {
        self.advance();
        self.write(Vec::new());
        Ok(())
    }

    fn serialize_some<T: ?Sized + serde::Serialize>(self, value: &T) -> SerializerResult<()> {
        value.serialize(self)
    }

    fn serialize_unit_struct(self, name: &'static str) -> SerializerResult<()> {
        self.advance_into(name);
        self.write(Vec::new());
        Ok(())
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> SerializerResult<()> {
        self.advance();
        self.write(variant.as_bytes());
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
        self.variant = None;

        self.advance();

        self.write(variant.as_bytes());
        self.output[self.file - 1].name = self.output[self.file - 2].name.clone();

        Ok(())
    }

    fn serialize_seq(self, _len: Option<usize>) -> SerializerResult<Self::SerializeSeq> {
        self.advance();
        self.write(Vec::new()); // Creates a folder
        self.output[self.file - 1].is_path = true;

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
        self.advance();
        self.variant = Some("value".into());
        self.write(variant.as_bytes());
        self.variant = None;

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
        self.advance();
        self.variant = Some("value".into());
        self.write(variant.as_bytes());
        self.variant = None;

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
        self.in_seq = true;
        value.serialize(&mut **self)?;

        Ok(())
    }

    fn end(self) -> SerializerResult<()> {
        self.in_seq = false;
        self.seq_n = 0;
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
        self.in_seq = true;
        value.serialize(&mut **self)
    }

    fn end(self) -> SerializerResult<()> {
        self.in_seq = false;
        self.seq_n = 0;
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
        self.in_seq = true;
        value.serialize(&mut **self)
    }

    fn end(self) -> SerializerResult<()> {
        self.in_seq = false;
        self.seq_n = 0;
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
        self.in_seq = true;
        value.serialize(&mut **self)
    }

    fn end(self) -> SerializerResult<()> {
        self.in_seq = false;
        self.seq_n = 0;
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
        self.is_key = true;
        key.serialize(&mut **self)
    }

    fn serialize_value<T>(&mut self, value: &T) -> SerializerResult<()>
    where
        T: ?Sized + serde::Serialize,
    {
        self.is_key = false;
        value.serialize(&mut **self)
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
        value.serialize(&mut **self)
    }

    fn end(self) -> SerializerResult<()> {
        Ok(())
    }
}
