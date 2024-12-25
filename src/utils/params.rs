extern crate num_bigint;

use std::{collections::BTreeMap, fmt::Debug};
use num_bigint::BigInt;

use base64::{Engine as _, engine::general_purpose};

#[derive(Clone, Debug, PartialEq)]
pub enum Params<'a> {
    Null,
    Boolean(bool),
    Integer(i64),
    BigInteger(BigInt),
    Decimal(f64),
    Text(String),
    ByteArray(&'a [u8]),
    Array(Vec<Params<'a>>),
    Dict(BTreeMap<String, Params<'a>>),
}

pub type QueryParams<'a> = Params<'a>;
pub type OperationParams<'a> = Params<'a>;

#[allow(dead_code)]
fn deserialize_bigint<'de, D>(deserializer: D) -> Result<BigInt, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let de_str: String = serde::Deserialize::deserialize(deserializer)?;
    BigInt::parse_bytes(de_str.as_bytes(), 10)
        .ok_or(serde::de::Error::custom("Failed to parse BigInt"))
}

#[derive(Clone, Debug, PartialEq)]
pub struct Operation<'a> {
    pub dict: Option<Vec<(&'a str, Params<'a>)>>,
    pub list: Option<Vec<Params<'a>>>,
    pub operation_name: Option<&'a str>,
}

impl<'a> Default for Operation<'a> {
    fn default() -> Self {
        Self {
            dict: None,
            list: None,
            operation_name: None,
        }
    }
}

impl<'a> Operation<'a> {
    pub fn from_dict(operation_name: &'a str, params: Vec<(&'a str, Params<'a>)>) -> Self {
        Self {
            dict: Some(params),
            operation_name: Some(operation_name),
            ..Default::default()
        }
    }

    pub fn from_list(operation_name: &'a str, params: Vec<Params<'a>>) -> Self {
        Self {
            list: Some(params),
            operation_name: Some(operation_name),
            ..Default::default()
        }
    }
}

impl<'a> Params<'a> {
    pub fn decimal_to_string(val: Box<f64>) -> String {
        val.to_string()
    }

    pub fn is_empty(self) -> bool {
        match self {
            Params::Array(array) => array.is_empty(),
            Params::Dict(dict) => dict.is_empty(),
            Params::ByteArray(bytearray) => bytearray.is_empty(),
            Params::Text(text) => text.is_empty(),
            _ => panic!("Cannot check empty of this type {:?}", self)
        }
    }

    pub fn len(self) -> usize {
        match self {
            Params::Array(array) => array.len(),
            Params::Dict(dict) => dict.len(),
            Params::ByteArray(bytearray) => bytearray.len(),
            Params::Text(text) => text.len(),
            _ => panic!("Cannot get length of this type {:?}", self)
        }
    }

    pub fn to_struct<T>(&self) -> Result<T, String>
    where
        T: Default + std::fmt::Debug + for<'de> serde::Deserialize<'de>,
    {
        match self {
            Params::Dict(_) => {
                let json_value = self.to_json_value();
                
                serde_json::from_value(json_value)
                    .map_err(|e| format!("Failed to convert Params to struct: {}", e))
            },
            _ => Err(format!("Expected Params::Dict, found {:?}", self)),
        }
    }

    pub fn to_json_value(&self) -> serde_json::Value {
        match self {
            Params::Null => serde_json::Value::Null,
            Params::Boolean(b) => serde_json::Value::Bool(*b),
            Params::Integer(i) => serde_json::Value::Number(serde_json::Number::from(*i)),
            Params::BigInteger(big_int) => {
                serde_json::Value::String(big_int.to_string())
            },
            Params::Decimal(d) => serde_json::Value::Number(serde_json::Number::from_f64(*d).unwrap()),
            Params::Text(text) => serde_json::Value::String(text.to_string()),
            Params::ByteArray(bytearray) => {
                let base64_encoded = general_purpose::STANDARD.encode(bytearray);
                serde_json::Value::String(base64_encoded)
            },
            Params::Array(array) => {
                let json_array: Vec<serde_json::Value> = array.iter().map(|param| param.to_json_value()).collect();
                serde_json::Value::Array(json_array)
            },
            Params::Dict(dict) => {
                let json_object: serde_json::Map<String, serde_json::Value> = dict.iter()
                    .map(|(key, value)| (key.to_string(), value.to_json_value()))
                    .collect();
                serde_json::Value::Object(json_object)
            },
        }
    }

    pub fn from_struct<T>(struct_instance: &T) -> Params<'a>
    where
        T: std::fmt::Debug + serde::Serialize,
    {
        let json_value = serde_json::to_value(struct_instance)
            .expect("Failed to convert struct to JSON value");

        Params::Dict(Self::json_value_to_params_dict(json_value))
    }

    fn json_value_to_params_dict(value: serde_json::Value) -> BTreeMap<String, Params<'a>> {
        let mut dict: BTreeMap<String, Params<'a>> = BTreeMap::new();

        if let serde_json::Value::Object(map) = value {
            for (key, val) in map {
                dict.insert(key, Self::value_to_params(val));
            }
        }

        dict
    }

    fn value_to_params(value: serde_json::Value) -> Params<'a> {
        match value {
            serde_json::Value::Null => Params::Null,
            serde_json::Value::Bool(b) => Params::Boolean(b),
            serde_json::Value::Number(n) => {
                if let Some(i) = n.as_i64() {
                    Params::Integer(i)
                } else if let Some(f) = n.as_f64() {
                    Params::Decimal(f)
                } else {
                    Params::Null
                }
            },
            serde_json::Value::String(s) => Params::Text(s),
            serde_json::Value::Array(arr) => {
                let params_array: Vec<Params> = arr.into_iter().map(Self::value_to_params).collect();
                Params::Array(params_array)
            },
            serde_json::Value::Object(_) => {
                Params::Null
            },
        }
    }

    #[cfg(debug_assertions)]
    pub fn debug_print(&self) {
        match self {
            Params::Array(array) => {
                    for item in array {
                        item.debug_print();
                    }
            } 
            Params::Dict(dict) => {
                    for item in dict {
                        eprintln!("key = {}", item.0);
                        eprintln!("value = ");
                        item.1.debug_print();
                    }
            }
            Params::ByteArray(val) => {
                eprintln!("{:?}", hex::encode(val));
            }
            _ =>
                eprintln!("{:?}", self)
        }
    }
}

impl<'a> Into<Vec<Params<'a>>> for Params<'a> {
    fn into(self) -> Vec<Params<'a>> {
        match self {
            Params::Array(array) => array,
            _ => panic!("Cannot convert {:?} into Vec<Params>", self),
        }
    }
}

impl<'a> Into<BTreeMap<String, Params<'a>>> for Params<'a> {
    fn into(self) -> BTreeMap<String, Params<'a>> {
        match self {
            Params::Dict(dict) => dict,
            _ => panic!("Cannot convert {:?} into BTreeMap", self),
        }
    }
}

#[test]
fn test_serialize_struct_to_param_dict() {
     #[derive(Debug, Default, serde::Serialize, PartialEq)]
    struct TestStruct1 {
        foo: String,
        bar: i64,
        ok: bool
    }

    let ts1 = TestStruct1 {
        foo: "".to_string(), bar: 1, ok: true
    };

    Params::from_struct(&ts1);
}

#[test]
fn test_deserialize_param_dict_to_struct() {
    /// We have two options here for deserialization big integer:
    /// 1. Use `String` struct
    /// 2. Use `num_bigint::BigInt` struct with serder custom function
    /// name `deserialize_bigint`
    #[derive(Debug, Default, serde::Deserialize, PartialEq)]
    struct TestNestedStruct {
        bigint_as_string: String,
        #[serde(deserialize_with = "deserialize_bigint")]
        bigint_as_num_bigint: num_bigint::BigInt
    }

    #[derive(Debug, Default, serde::Deserialize, PartialEq)]
    struct TestStruct {
        x: i64,
        y: i64,
        z: String,
        l: bool,
        n: f64,
        m: String,
        dict: TestNestedStruct,
        array: Vec<serde_json::Value>
    }

    let bigint = num_bigint::BigInt::from(100000000000000000000000 as i128);
    let bytearray_value = b"1234";
    let bytearray_base64_encoded = general_purpose::STANDARD.encode(bytearray_value);

    let ts = TestStruct{
        x: 1, y: 2, z: "foo".to_string(), dict: TestNestedStruct {
            bigint_as_string: bigint.to_string(),
            bigint_as_num_bigint: (100000000000000000000000 as i128).into()
        }, l: true, n: 3.14, m: bytearray_base64_encoded, array: vec![
            serde_json::Value::Number(serde_json::Number::from(1 as i64)),
            serde_json::Value::String("foo".to_string()),
            ]
    };

    let mut nested_params: BTreeMap<String, Params> = BTreeMap::new();
    nested_params.insert("bigint_as_string".to_string(), Params::BigInteger(bigint.clone()));
    nested_params.insert("bigint_as_num_bigint".to_string(), Params::BigInteger(bigint.clone()));

    let mut params: BTreeMap<String, Params> = BTreeMap::new();
    params.insert("x".to_string(), Params::Integer(1));
    params.insert("y".to_string(), Params::Integer(2));
    params.insert("z".to_string(), Params::Text("foo".to_string()));
    params.insert("dict".to_string(), Params::Dict(nested_params));
    params.insert("l".to_string(), Params::Boolean(true));
    params.insert("n".to_string(), Params::Decimal(3.14));
    params.insert("m".to_string(), Params::ByteArray(bytearray_value));
    params.insert("array".to_string(), Params::Array(vec![Params::Integer(1), Params::Text("foo".to_string())]));

    let params_dict = Params::Dict(params);
    let result: Result<TestStruct, String> = params_dict.to_struct();

    if let Ok(val) = result {
        assert_eq!(ts, val);
    } else {
        panic!("Error deserializing params: {}", result.unwrap_err());
    }
}