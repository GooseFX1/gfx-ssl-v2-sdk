use serde::ser::{self, Serialize, SerializeStruct, Serializer};
use std::fmt;

type Result<T> = std::result::Result<T, fmt::Error>;

// Two spaces default
const DEFAULT_INDENT: &'static str = "  ";

pub fn cli_pretty_print<T>(value: &T) -> String
where
    T: Serialize,
{
    let mut serializer = CliPrettyPrinter::default();
    value.serialize(&mut serializer).unwrap();
    serializer.output
}

pub struct CliPrettyPrinter {
    // Output is stored as a String
    output: String,
    // Current indentation level
    indent: usize,
    // Indentation string to repeat
    indent_str: String,
}

impl Default for CliPrettyPrinter {
    fn default() -> Self {
        Self {
            output: Default::default(),
            indent: Default::default(),
            indent_str: DEFAULT_INDENT.to_string(),
        }
    }
}

impl CliPrettyPrinter {
    #[allow(dead_code)]
    fn with_indent(indent_str: String) -> CliPrettyPrinter {
        CliPrettyPrinter {
            output: String::new(),
            indent: 0,
            indent_str,
        }
    }

    fn indent(&self) -> String {
        self.indent_str.repeat(self.indent)
    }
}

impl<'a> Serializer for &'a mut CliPrettyPrinter {
    type Ok = ();
    type Error = fmt::Error;
    type SerializeSeq = Self;
    type SerializeTuple = Self;
    type SerializeTupleStruct = Self;
    type SerializeTupleVariant = Self;
    type SerializeMap = Self;
    type SerializeStruct = Self;
    type SerializeStructVariant = Self;

    fn serialize_bool(self, v: bool) -> Result<()> {
        self.output += if v { "true" } else { "false" };
        Ok(())
    }

    fn serialize_i8(self, v: i8) -> Result<()> {
        self.serialize_i64(i64::from(v))
    }

    fn serialize_i16(self, v: i16) -> Result<()> {
        self.serialize_i64(i64::from(v))
    }

    fn serialize_i32(self, v: i32) -> Result<()> {
        self.serialize_i64(i64::from(v))
    }

    fn serialize_i64(self, v: i64) -> Result<()> {
        self.output += &v.to_string();
        Ok(())
    }

    fn serialize_u8(self, v: u8) -> Result<()> {
        self.serialize_u64(u64::from(v))
    }

    fn serialize_u16(self, v: u16) -> Result<()> {
        self.serialize_u64(u64::from(v))
    }

    fn serialize_u32(self, v: u32) -> Result<()> {
        self.serialize_u64(u64::from(v))
    }

    fn serialize_u64(self, v: u64) -> Result<()> {
        self.output += &v.to_string();
        Ok(())
    }

    fn serialize_f32(self, v: f32) -> Result<()> {
        self.serialize_f64(f64::from(v))
    }

    fn serialize_f64(self, v: f64) -> Result<()> {
        self.output += &v.to_string();
        Ok(())
    }

    fn serialize_char(self, v: char) -> Result<()> {
        self.serialize_str(&v.to_string())
    }

    fn serialize_str(self, v: &str) -> Result<()> {
        self.output += "\"";
        self.output += v;
        self.output += "\"";
        Ok(())
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<()> {
        use serde::ser::SerializeSeq;
        let mut seq = self.serialize_seq(Some(v.len()))?;
        for byte in v {
            seq.serialize_element(byte)?;
        }
        SerializeSeq::end(seq)
    }

    fn serialize_none(self) -> Result<()> {
        self.serialize_unit()
    }

    fn serialize_some<T>(self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    fn serialize_unit(self) -> Result<()> {
        self.output += "null";
        Ok(())
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<()> {
        self.serialize_unit()
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<()> {
        self.serialize_str(variant)
    }

    fn serialize_newtype_struct<T>(self, _name: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T>(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        self.output += "{";
        variant.serialize(&mut *self)?;
        self.output += ":";
        value.serialize(&mut *self)?;
        self.output += "}";
        Ok(())
    }

    fn serialize_struct(self, _name: &'static str, _: usize) -> Result<Self::SerializeStruct> {
        // This intentionally doesn't print the name of the struct,
        // we are either relying on a field name,
        // and/or the assumption that the struct name is implicit or not relevant to UI.
        if self.indent != 0 {
            self.output += "\n";
        }
        self.indent += 1;
        Ok(self)
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq> {
        self.output += "[";
        Ok(self)
    }

    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple> {
        self.serialize_seq(Some(len))
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct> {
        self.serialize_seq(Some(len))
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleVariant> {
        self.output += &format!("{}: ", variant);
        self.indent += 1;
        self.serialize_seq(Some(len))
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap> {
        self.output += "\n";
        self.indent += 1;
        Ok(self)
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant> {
        self.output += &format!("{}: \n", variant);
        self.indent += 1;
        Ok(self)
    }
}

impl<'a> SerializeStruct for &'a mut CliPrettyPrinter {
    type Ok = ();
    type Error = fmt::Error;

    fn serialize_field<T: ?Sized>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: Serialize,
    {
        self.output += &format!("{}{}: ", self.indent(), key);
        value.serialize(&mut **self)?;
        if !self.output.ends_with("\n") {
            self.output += "\n";
        }
        Ok(())
    }

    fn end(self) -> Result<Self::Ok> {
        if !self.output.ends_with("\n") {
            self.output += "\n";
        }
        self.indent -= 1;
        Ok(())
    }
}

impl<'a> ser::SerializeMap for &'a mut CliPrettyPrinter {
    type Ok = ();
    type Error = fmt::Error;

    fn serialize_key<T>(&mut self, key: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        key.serialize(&mut **self)
    }

    // It doesn't make a difference whether the colon is printed at the end of
    // `serialize_key` or at the beginning of `serialize_value`. In this case
    // the code is a bit simpler having it here.
    fn serialize_value<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(&mut **self)?;
        if !self.output.ends_with("\n") {
            self.output += "\n";
        }
        Ok(())
    }

    fn end(self) -> Result<()> {
        if !self.output.ends_with("\n") {
            self.output += "\n";
        }
        self.indent -= 1;
        Ok(())
    }
}

impl<'a> ser::SerializeStructVariant for &'a mut CliPrettyPrinter {
    type Ok = ();
    type Error = fmt::Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        self.output += &format!("{}{}: ", self.indent(), key);
        value.serialize(&mut **self)?;
        self.output += "\n";
        Ok(())
    }

    fn end(self) -> Result<()> {
        self.indent -= 1;
        Ok(())
    }
}

impl<'a> ser::SerializeSeq for &'a mut CliPrettyPrinter {
    type Ok = ();
    type Error = fmt::Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        if !self.output.ends_with('[') {
            self.output += ", ";
        }
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<()> {
        self.output += "]";
        Ok(())
    }
}

impl<'a> ser::SerializeTuple for &'a mut CliPrettyPrinter {
    type Ok = ();
    type Error = fmt::Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        if !self.output.ends_with('[') {
            self.output += ", ";
        }
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<()> {
        self.output += "]";
        Ok(())
    }
}

impl<'a> ser::SerializeTupleStruct for &'a mut CliPrettyPrinter {
    type Ok = ();
    type Error = fmt::Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        if !self.output.ends_with('[') {
            self.output += ", ";
        }
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<()> {
        self.output += "]";
        Ok(())
    }
}

impl<'a> ser::SerializeTupleVariant for &'a mut CliPrettyPrinter {
    type Ok = ();
    type Error = fmt::Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        if !self.output.ends_with('[') {
            self.output += ", ";
        }
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<()> {
        self.output += "]";
        self.indent -= 1;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Serialize;

    #[derive(Serialize)]
    enum MyEnum {
        Variant1(String, u64),
        Variant2,
        Variant3 { subfield1: f64, subfield2: String },
    }

    #[derive(Serialize)]
    struct MyStruct {
        field1: String,
        field2: MySubStruct,
        field3: i64,
    }

    #[derive(Serialize)]
    struct MySubStruct {
        field1: String,
        field2: [u8; 4],
        field3: MyEnum,
        field4: MyEnum,
        field5: MyEnum,
        field6: (String, String),
    }

    #[test]
    fn test_pretty_print() {
        let my_struct = MyStruct {
            field1: "Hello".to_string(),
            field2: MySubStruct {
                field1: "Foo".to_string(),
                field2: [8, 9, 10, 11],
                field3: MyEnum::Variant1("Bar".to_string(), 999),
                field4: MyEnum::Variant2,
                field5: MyEnum::Variant3 {
                    subfield1: 12383.47,
                    subfield2: "Baz".to_string(),
                },
                field6: ("Bang".to_string(), "Boo".to_string()),
            },
            field3: -444,
        };

        println!(
            "Serialized output (twice to check newlines):\n{}{}",
            cli_pretty_print(&my_struct),
            cli_pretty_print(&my_struct),
        );
    }
}
