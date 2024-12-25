use crate::utils::params::Params;
use std::collections::BTreeMap;

pub trait ToParam {
    fn to_param(&self) -> Params<'_>;
}

impl ToParam for i8 {
    fn to_param(&self) -> Params<'_> {
        Params::Integer((*self).into())
    }
}

impl ToParam for i16 {
    fn to_param(&self) -> Params<'_> {
        Params::Integer((*self).into())
    }
}

impl ToParam for i32 {
    fn to_param(&self) -> Params<'_> {
        Params::Integer((*self).into())
    }
}

impl ToParam for i64 {
    fn to_param(&self) -> Params<'_> {
        Params::Integer((*self).into())
    }
}

impl ToParam for i128 {
    fn to_param(&self) -> Params<'_> {
        Params::BigInteger((*self).into())
    }
}

impl ToParam for f64 {
    fn to_param(&self) -> Params<'_> {
        Params::Decimal((*self).into())
    }
}

impl ToParam for String {
    fn to_param(&self) -> Params<'_> {
        Params::Text(self.to_string())
    }
}

impl ToParam for &str {
    fn to_param(&self) -> Params<'_> {
        Params::Text(self.to_string())
    }
}

impl ToParam for bool {
    fn to_param(&self) -> Params<'_> {
        Params::Boolean((*self).into())
    }
}

impl ToParam for Vec<u8> {
    fn to_param(&self) -> Params<'_> {
        Params::ByteArray(self)
    }
}

impl<T: ToParam> ToParam for Vec<T> {
    fn to_param(&self) -> Params<'_> {
        let params_array: Vec<Params<'_>> = self.iter().map(|item| item.to_param()).collect();
        Params::Array(params_array)
    }
}

impl<'a, T: ToParam> ToParam for BTreeMap<&'a str, T> {
    fn to_param(&self) -> Params<'_> {
        let dict: BTreeMap<String, Params<'_>> = self.iter()
            .map(|(key, value)| (key.to_string(), value.to_param())) 
            .collect();
        Params::Dict(dict)
    }
}

impl<T: ToParam> ToParam for BTreeMap<String, T> {
    fn to_param(&self) -> Params<'_> {
        let dict: BTreeMap<String, Params<'_>> = self.iter()
            .map(|(key, value)| (key.clone(), value.to_param()))
            .collect();
        Params::Dict(dict)
    }
}

#[test]
fn test_integer_to_param() {
  let i: i32 = 1;
  let r = i.to_param();
  assert_eq!(Params::Integer(i.into()), r);

  let ii: i128 = 1234567890;
  let r = ii.to_param();
  assert_eq!(Params::BigInteger(ii.into()), r);
}

#[test]
fn test_decimal_to_param() {
  let f: f64 = 1.234;
  let r = f.to_param();
  assert_eq!(Params::Decimal(f.into()), r);
}

#[test]
fn test_string_to_param() {
  let s: String = "Hello!".to_string();
  let r = s.to_param();
  assert_eq!(Params::Text(s.clone()), r);

  let ss: &str = "Hello!";
  let r = ss.to_param();
  assert_eq!(Params::Text(ss.to_string()), r);
}

#[test]
fn test_bool_to_param() {
    let b = true;
    let r = b.to_param();
    assert_eq!(Params::Boolean(b.into()), r);
}

#[test]
fn test_bytearray_to_param() {
    let ba = b"123456".to_vec();
    let r = ba.to_param();
    assert_eq!(Params::ByteArray(&ba), r);
}

#[test]
fn test_struct_fileds_to_params() {
    struct TestStruct {
        name: String,
        array: Vec<String>,
        brid: Vec<u8>,
        dict: BTreeMap<String, BTreeMap<String, String>>,
    }

    let dict_val = BTreeMap::from([("key1".to_string(), BTreeMap::from([("key2".to_string(), "value".to_string())]))]);

    let test_struct = TestStruct {
        name: "foo".to_string(),
        array: vec!["arg1".to_string(), "arg2".to_string()],
        dict: dict_val.clone(),
        brid: b"1234".to_vec()
    };

    let t1 = test_struct.name.to_param();
    let t2 = test_struct.array.to_param();
    let t3 = test_struct.dict.to_param();
    let t4 = test_struct.brid.to_param();

    assert_eq!(Params::Text("foo".to_string()), t1);
    assert_eq!(Params::Array(vec![Params::Text("arg1".to_string()), Params::Text("arg2".to_string())]), t2);
    assert_eq!(dict_val.to_param(), t3);
    assert_eq!(Params::ByteArray(b"1234"), t4);
}