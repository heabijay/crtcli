use serde_json::ser::{CharEscape, Formatter};
use std::io::Write;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct JsonMsDatePreserveFormatter<F>
where
    F: Formatter,
{
    inner_formatter: F,
}

impl<F: Formatter> JsonMsDatePreserveFormatter<F> {
    pub fn new(inner_formatter: F) -> Self {
        Self { inner_formatter }
    }
}

#[allow(dead_code)]
impl JsonMsDatePreserveFormatter<serde_json::ser::CompactFormatter> {
    pub fn new_compact() -> Self {
        Self::new(serde_json::ser::CompactFormatter)
    }
}

impl JsonMsDatePreserveFormatter<serde_json::ser::PrettyFormatter<'_>> {
    pub fn new_pretty() -> Self {
        Self::new(serde_json::ser::PrettyFormatter::new())
    }
}

impl<F: Formatter> Formatter for JsonMsDatePreserveFormatter<F> {
    fn write_null<W>(&mut self, writer: &mut W) -> std::io::Result<()>
    where
        W: ?Sized + Write,
    {
        self.inner_formatter.write_null(writer)
    }

    fn write_bool<W>(&mut self, writer: &mut W, value: bool) -> std::io::Result<()>
    where
        W: ?Sized + Write,
    {
        self.inner_formatter.write_bool(writer, value)
    }

    fn write_i8<W>(&mut self, writer: &mut W, value: i8) -> std::io::Result<()>
    where
        W: ?Sized + Write,
    {
        self.inner_formatter.write_i8(writer, value)
    }

    fn write_i16<W>(&mut self, writer: &mut W, value: i16) -> std::io::Result<()>
    where
        W: ?Sized + Write,
    {
        self.inner_formatter.write_i16(writer, value)
    }

    fn write_i32<W>(&mut self, writer: &mut W, value: i32) -> std::io::Result<()>
    where
        W: ?Sized + Write,
    {
        self.inner_formatter.write_i32(writer, value)
    }

    fn write_i64<W>(&mut self, writer: &mut W, value: i64) -> std::io::Result<()>
    where
        W: ?Sized + Write,
    {
        self.inner_formatter.write_i64(writer, value)
    }

    fn write_i128<W>(&mut self, writer: &mut W, value: i128) -> std::io::Result<()>
    where
        W: ?Sized + Write,
    {
        self.inner_formatter.write_i128(writer, value)
    }

    fn write_u8<W>(&mut self, writer: &mut W, value: u8) -> std::io::Result<()>
    where
        W: ?Sized + Write,
    {
        self.inner_formatter.write_u8(writer, value)
    }

    fn write_u16<W>(&mut self, writer: &mut W, value: u16) -> std::io::Result<()>
    where
        W: ?Sized + Write,
    {
        self.inner_formatter.write_u16(writer, value)
    }

    fn write_u32<W>(&mut self, writer: &mut W, value: u32) -> std::io::Result<()>
    where
        W: ?Sized + Write,
    {
        self.inner_formatter.write_u32(writer, value)
    }

    fn write_u64<W>(&mut self, writer: &mut W, value: u64) -> std::io::Result<()>
    where
        W: ?Sized + Write,
    {
        self.inner_formatter.write_u64(writer, value)
    }

    fn write_u128<W>(&mut self, writer: &mut W, value: u128) -> std::io::Result<()>
    where
        W: ?Sized + Write,
    {
        self.inner_formatter.write_u128(writer, value)
    }

    fn write_f32<W>(&mut self, writer: &mut W, value: f32) -> std::io::Result<()>
    where
        W: ?Sized + Write,
    {
        self.inner_formatter.write_f32(writer, value)
    }

    fn write_f64<W>(&mut self, writer: &mut W, value: f64) -> std::io::Result<()>
    where
        W: ?Sized + Write,
    {
        self.inner_formatter.write_f64(writer, value)
    }

    fn write_number_str<W>(&mut self, writer: &mut W, value: &str) -> std::io::Result<()>
    where
        W: ?Sized + Write,
    {
        self.inner_formatter.write_number_str(writer, value)
    }

    fn begin_string<W>(&mut self, writer: &mut W) -> std::io::Result<()>
    where
        W: ?Sized + Write,
    {
        self.inner_formatter.begin_string(writer)
    }

    fn end_string<W>(&mut self, writer: &mut W) -> std::io::Result<()>
    where
        W: ?Sized + Write,
    {
        self.inner_formatter.end_string(writer)
    }

    fn write_string_fragment<W>(&mut self, writer: &mut W, fragment: &str) -> std::io::Result<()>
    where
        W: ?Sized + Write,
    {
        if fragment.starts_with("/Date(") && fragment.ends_with(")/") {
            let timestamp = &fragment[6..(fragment.len() - 2)];

            // \/Date(1729879200000+0300)\/
            let is_timestamp_valid = timestamp.chars().all(|c| c.is_ascii_digit() || c == '+');

            if is_timestamp_valid {
                return self
                    .inner_formatter
                    .write_string_fragment(writer, &format!("\\/Date({timestamp})\\/"));
            }
        }

        self.inner_formatter.write_string_fragment(writer, fragment)
    }

    fn write_char_escape<W>(
        &mut self,
        writer: &mut W,
        char_escape: CharEscape,
    ) -> std::io::Result<()>
    where
        W: ?Sized + Write,
    {
        self.inner_formatter.write_char_escape(writer, char_escape)
    }

    fn write_byte_array<W>(&mut self, writer: &mut W, value: &[u8]) -> std::io::Result<()>
    where
        W: ?Sized + Write,
    {
        self.inner_formatter.write_byte_array(writer, value)
    }

    fn begin_array<W>(&mut self, writer: &mut W) -> std::io::Result<()>
    where
        W: ?Sized + Write,
    {
        self.inner_formatter.begin_array(writer)
    }

    fn end_array<W>(&mut self, writer: &mut W) -> std::io::Result<()>
    where
        W: ?Sized + Write,
    {
        self.inner_formatter.end_array(writer)
    }

    fn begin_array_value<W>(&mut self, writer: &mut W, first: bool) -> std::io::Result<()>
    where
        W: ?Sized + Write,
    {
        self.inner_formatter.begin_array_value(writer, first)
    }

    fn end_array_value<W>(&mut self, _writer: &mut W) -> std::io::Result<()>
    where
        W: ?Sized + Write,
    {
        self.inner_formatter.end_array_value(_writer)
    }

    fn begin_object<W>(&mut self, writer: &mut W) -> std::io::Result<()>
    where
        W: ?Sized + Write,
    {
        self.inner_formatter.begin_object(writer)
    }

    fn end_object<W>(&mut self, writer: &mut W) -> std::io::Result<()>
    where
        W: ?Sized + Write,
    {
        self.inner_formatter.end_object(writer)
    }

    fn begin_object_key<W>(&mut self, writer: &mut W, first: bool) -> std::io::Result<()>
    where
        W: ?Sized + Write,
    {
        self.inner_formatter.begin_object_key(writer, first)
    }

    fn end_object_key<W>(&mut self, _writer: &mut W) -> std::io::Result<()>
    where
        W: ?Sized + Write,
    {
        self.inner_formatter.end_object_key(_writer)
    }

    fn begin_object_value<W>(&mut self, writer: &mut W) -> std::io::Result<()>
    where
        W: ?Sized + Write,
    {
        self.inner_formatter.begin_object_value(writer)
    }

    fn end_object_value<W>(&mut self, _writer: &mut W) -> std::io::Result<()>
    where
        W: ?Sized + Write,
    {
        self.inner_formatter.end_object_value(_writer)
    }

    fn write_raw_fragment<W>(&mut self, writer: &mut W, fragment: &str) -> std::io::Result<()>
    where
        W: ?Sized + Write,
    {
        self.inner_formatter.write_raw_fragment(writer, fragment)
    }

    //#endregion
}
