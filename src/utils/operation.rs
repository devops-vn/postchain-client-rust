//! Operation parameter handling and data type conversion utilities.
//! 
//! This module provides functionality for handling operation parameters,
//! data type conversions, and serialization/deserialization of various
//! data types used in blockchain operations.
//! 
//! # Features
//! - Generic parameter type system supporting various data types
//! - Conversion between Rust structs and operation parameters
//! - Serialization/deserialization support for complex data types
//! - Support for large integers and binary data
//! 
//! # Example
//! ```
//! use std::collections::BTreeMap;
//! use crate::utils::operation::{Operation, Params};
//! 
//! // Create operation parameters
//! let params = vec![
//!     ("key", Params::Text("value".to_string())),
//!     ("number", Params::Integer(42))
//! ];
//! 
//! // Create an operation
//! let operation = Operation::from_dict("my_operation", params);
//! ```

extern crate num_bigint;

use std::{collections::BTreeMap, fmt::Debug};
use num_bigint::BigInt;
use base64::{Engine as _, engine::general_purpose};

/// Represents different types of operation parameters.
/// 
/// This enum provides a type-safe way to handle various data types
/// used in blockchain operations, including primitive types, collections,
/// and special types like BigInteger.
#[derive(Clone, Debug, PartialEq)]
pub enum Params {
    /// Represents a null value
    Null,
    /// Represents a boolean value (true/false)
    Boolean(bool),
    /// Represents a 64-bit signed integer
    Integer(i64),
    /// Represents an arbitrary-precision integer using BigInt
    BigInteger(BigInt),
    /// Represents a 64-bit floating point number
    Decimal(f64),
    /// Represents a UTF-8 encoded string
    Text(String),
    /// Represents a raw byte array
    ByteArray(Vec<u8>),
    /// Represents an ordered collection of Params
    Array(Vec<Params>),
    /// Represents a key-value mapping where keys are strings
    Dict(BTreeMap<String, Params>)
}

pub type QueryParams = Params;
pub type OperationParams = Params;

/// Deserializes a string into a BigInt.
/// 
/// This function is used with serde to deserialize string-encoded
/// big integers into BigInt type.
/// 
/// # Arguments
/// * `deserializer` - The deserializer to use
/// 
/// # Returns
/// Result containing either the deserialized BigInt or an error
#[allow(dead_code)]
fn deserialize_bigint<'de, D>(deserializer: D) -> Result<BigInt, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let de_str: String = serde::Deserialize::deserialize(deserializer)?;
    
    BigInt::parse_bytes(de_str.as_bytes(), 10)
        .ok_or(serde::de::Error::custom("Failed to parse BigInt"))
}

/// Deserializes a base64 string into a byte array.
/// 
/// This function is used with serde to deserialize base64-encoded
/// strings into byte arrays.
/// 
/// # Arguments
/// * `deserializer` - The deserializer to use
/// 
/// # Returns
/// Result containing either the deserialized byte array or an error
#[allow(dead_code)]
fn deserialize_byte_array<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let base64_str: String = serde::Deserialize::deserialize(deserializer)?;
    general_purpose::STANDARD.decode(&base64_str).map_err(serde::de::Error::custom)
}


/// Serializes a BigInt into a string.
/// 
/// This function is used with serde to serialize BigInt values
/// into string format.
/// 
/// # Arguments
/// * `bigint` - The BigInt to serialize
/// * `serializer` - The serializer to use
/// 
/// # Returns
/// Result containing either the serialized string or an error
#[allow(dead_code)]
fn serialize_bigint<S>(bigint: &BigInt, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    let bigint_str = bigint.to_string();
    serializer.serialize_str(&bigint_str)
}

/// Represents a blockchain operation with parameters.
/// 
/// An operation can contain either a dictionary of named parameters
/// or a list of unnamed parameters, along with an operation name.
#[derive(Clone, Debug, PartialEq)]
pub struct Operation<'a> {
    /// Dictionary of named parameters
    /// List of unnamed parameters
    /// Name of the operation
    pub dict: Option<Vec<(&'a str, Params)>>,
    pub list: Option<Vec<Params>>,
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

/// Checks if a vector of JSON values represents a byte array.
/// 
/// # Arguments
/// * `value` - Vector of JSON values to check
/// 
/// # Returns
/// true if all values are valid u8 numbers
fn is_vec_u8(value: &Vec<serde_json::Value>) -> bool {
    value.iter().all(|v| {
            if let serde_json::Value::Number(n) = v {
                n.is_u64() && n.as_u64().unwrap() <= u8::MAX as u64
            } else {
                false
            }
        })    
}

impl<'a> Operation<'a> {
    /// Creates a new Operation from a dictionary of parameters.
    /// 
    /// # Arguments
    /// * `operation_name` - Name of the operation
    /// * `params` - Vector of key-value parameter pairs
    /// 
    /// # Returns
    /// A new Operation instance with dictionary parameters
    pub fn from_dict(operation_name: &'a str, params: Vec<(&'a str, Params)>) -> Self {
        Self {
            dict: Some(params),
            operation_name: Some(operation_name),
            ..Default::default()
        }
    }

    /// Creates a new Operation from a list of parameters.
    /// 
    /// # Arguments
    /// * `operation_name` - Name of the operation
    /// * `params` - Vector of parameters
    /// 
    /// # Returns
    /// A new Operation instance with list parameters
    pub fn from_list(operation_name: &'a str, params: Vec<Params>) -> Self {
        Self {
            list: Some(params),
            operation_name: Some(operation_name),
            ..Default::default()
        }
    }
}

impl Params {
    /// Converts a boxed f64 value to its string representation.
    /// 
    /// # Arguments
    /// * `val` - Boxed f64 value to convert
    /// 
    /// # Returns
    /// String representation of the decimal value
    pub fn decimal_to_string(val: Box<f64>) -> String {
        val.to_string()
    }

    /// Converts a dictionary parameter to an array of its values.
    /// 
    /// # Arguments
    /// * `self` - Dictionary parameter to convert
    /// 
    /// # Returns
    /// Vector containing the values from the dictionary
    /// 
    /// # Panics
    /// Panics if self is not a Params::Dict
    pub fn dict_to_array(self) -> Vec<Params> {
        match self {
            Params::Dict(dict) => {
                let values: Vec<Params> = dict.into_iter()
                    .filter_map(|(_, value)| {
                        Some(value)
                    })
                    .collect();
                values
            },
            _ => panic!("Expected Params::Dict, found {:?}", self),
        }
    }

    /// Checks if the parameter value is empty.
    /// 
    /// Works with Array, Dict, ByteArray, and Text parameter types.
    /// 
    /// # Returns
    /// true if the parameter value is empty
    /// 
    /// # Panics
    /// Panics if called on parameter types that don't support emptiness check
    pub fn is_empty(self) -> bool {
        match self {
            Params::Array(array) => array.is_empty(),
            Params::Dict(dict) => dict.is_empty(),
            Params::ByteArray(bytearray) => bytearray.is_empty(),
            Params::Text(text) => text.is_empty(),
            _ => panic!("Cannot check empty of this type {:?}", self)
        }
    }

    /// Returns the length of the parameter value.
    /// 
    /// Works with Array, Dict, ByteArray, and Text parameter types.
    /// 
    /// # Returns
    /// Length of the parameter value
    /// 
    /// # Panics
    /// Panics if called on parameter types that don't support length
    pub fn len(self) -> usize {
        match self {
            Params::Array(array) => array.len(),
            Params::Dict(dict) => dict.len(),
            Params::ByteArray(bytearray) => bytearray.len(),
            Params::Text(text) => text.len(),
            _ => panic!("Cannot get length of this type {:?}", self)
        }
    }

    /// Converts a dictionary parameter to a Rust struct.
    /// 
    /// # Type Parameters
    /// * `T` - The target struct type that implements Default + Debug + Deserialize
    /// 
    /// # Returns
    /// Result containing either the converted struct or an error message
    /// 
    /// # Example
    /// ```
    /// #[derive(Debug, Default, serde::Deserialize)]
    /// struct MyStruct {
    ///     field: String,
    ///     value: i64,
    /// }
    /// 
    /// let dict = Params::Dict(/* ... */);
    /// let result: Result<MyStruct, String> = dict.to_struct();
    /// ```
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

    /// Converts the parameter to a serde_json::Value.
    /// 
    /// This method handles all parameter types, including complex types
    /// like BigInteger and ByteArray.
    /// 
    /// # Returns
    /// JSON representation of the parameter
    pub fn to_json_value(&self) -> serde_json::Value {
        match *self {
            Params::Null => serde_json::Value::Null,
            Params::Boolean(b) => serde_json::Value::Bool(b),
            Params::Integer(i) => serde_json::Value::Number(serde_json::Number::from(i)),
            Params::BigInteger(ref big_int) => {
                serde_json::Value::String(big_int.to_string())
            },
            Params::Decimal(d) => serde_json::Value::Number(serde_json::Number::from_f64(d).unwrap()),
            Params::Text(ref text) => serde_json::Value::String(text.to_string()),
            Params::ByteArray(ref bytearray) => {
                let base64_encoded = general_purpose::STANDARD.encode(bytearray);
                serde_json::Value::String(base64_encoded)
            },
            Params::Array(ref array) => {
                let json_array: Vec<serde_json::Value> = array.iter().map(|param| param.to_json_value()).collect();
                serde_json::Value::Array(json_array)
            },
            Params::Dict(ref dict) => {
                let json_object: serde_json::Map<String, serde_json::Value> = dict.iter()
                    .map(|(key, value)| (key.to_string(), value.to_json_value()))
                    .collect();
                serde_json::Value::Object(json_object)
            },
        }
    }

    /// Creates a parameter from a Rust struct.
    /// 
    /// # Type Parameters
    /// * `T` - The source struct type that implements Debug + Serialize
    /// 
    /// # Arguments
    /// * `struct_instance` - Reference to the struct to convert
    /// 
    /// # Returns
    /// Dictionary parameter containing the struct's fields
    /// 
    /// # Example
    /// ```
    /// #[derive(Debug, serde::Serialize)]
    /// struct MyStruct {
    ///     field: String,
    ///     value: i64,
    /// }
    /// 
    /// let my_struct = MyStruct { field: "test".into(), value: 42 };
    /// let params = Params::from_struct(&my_struct);
    /// ```
    pub fn from_struct<T>(struct_instance: &T) -> Params
    where
        T: std::fmt::Debug + serde::Serialize,
    {
        let json_value = serde_json::to_value(struct_instance)
            .expect("Failed to convert struct to JSON value");

        Params::Dict(Self::json_value_to_params_dict(json_value))
    }

    /// Converts a JSON value to a parameter dictionary.
    /// 
    /// # Arguments
    /// * `value` - JSON value to convert
    /// 
    /// # Returns
    /// BTreeMap containing the converted parameters
    fn json_value_to_params_dict(value: serde_json::Value) -> BTreeMap<String, Params> {
        let mut dict: BTreeMap<String, Params> = BTreeMap::new();

        if let serde_json::Value::Object(map) = value {
            for (key, val) in map {
                dict.insert(key, Self::value_to_params(val));
            }
        }

        dict
    }

    /// Creates a list of parameters from a Rust struct.
    /// 
    /// Similar to from_struct, but returns a vector of values
    /// instead of a dictionary.
    /// 
    /// # Type Parameters
    /// * `T` - The source struct type that implements Debug + Serialize
    /// 
    /// # Arguments
    /// * `struct_instance` - Reference to the struct to convert
    /// 
    /// # Returns
    /// Vector of parameters containing the struct's field values
    pub fn from_struct_to_list<T>(struct_instance: &T) -> Vec<Params>
    where
        T: std::fmt::Debug + serde::Serialize,
    {
        let json_value = serde_json::to_value(struct_instance)
            .expect("Failed to convert struct to JSON value");

        let mut vec = Vec::new();

        if let serde_json::Value::Object(map) = json_value {
            for (_, val) in map {
                vec.push(Self::value_to_params(val));
            }
        }

        vec
    }

    /// Converts a JSON value to a parameter.
    /// 
    /// This function handles conversion of various JSON types to
    /// their corresponding parameter types.
    /// 
    /// # Arguments
    /// * `value` - JSON value to convert
    /// 
    /// # Returns
    /// Converted parameter
    fn value_to_params(value: serde_json::Value) -> Params {
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
            serde_json::Value::String(s) => {
                match BigInt::parse_bytes(s.as_bytes(), 10) {
                    Some(big_int) => Params::BigInteger(big_int),
                    None => Params::Text(s),
                }
            },
            serde_json::Value::Array(arr) => {
                let is_vec_u8 = is_vec_u8(&arr);
                if is_vec_u8 {
                    let barr: Vec<u8> = arr.iter().map(|v|{v.as_u64().unwrap() as u8}).collect();
                    return Params::ByteArray(barr)
                }
                let params_array: Vec<Params> = arr.into_iter().map(Self::value_to_params).collect();
                Params::Array(params_array)
            },
            serde_json::Value::Object(dict) => {
                let params_dict: BTreeMap<String, Params> = dict.into_iter().map(|(k, v)| ( k, Self::value_to_params(v))).collect();
                Params::Dict(params_dict)
            }
        }
    }

    /// Prints debug information about the parameter.
    /// 
    /// This method is only available in debug builds and provides
    /// detailed information about the parameter's content.
    /// 
    /// # Arguments
    /// * `self` - The parameter to debug print
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

/// Implements conversion from Params to `Vec<Params>`.
/// 
/// This implementation allows converting an Array parameter
/// into a vector of parameters.
/// 
/// # Panics
/// Panics if the parameter is not an Array type
impl<'a> Into<Vec<Params>> for Params {
    fn into(self) -> Vec<Params> {
        match self {
            Params::Array(array) => array,
            _ => panic!("Cannot convert {:?} into Vec<Params>", self),
        }
    }
}

/// Implements conversion from Params to BTreeMap<String, Params>.
/// 
/// This implementation allows converting a Dict parameter
/// into a BTreeMap of string keys and parameter values.
/// 
/// # Panics
/// Panics if the parameter is not a Dict type
impl<'a> Into<BTreeMap<String, Params>> for Params {
    fn into(self) -> BTreeMap<String, Params> {
        match self {
            Params::Dict(dict) => dict,
            _ => panic!("Cannot convert {:?} into BTreeMap", self),
        }
    }
}

#[test]
fn test_serialize_struct_to_param_dict() {
    #[derive(Debug, Default, serde::Serialize, serde::Deserialize, PartialEq)]
    struct TestStruct2 {
        foo: String
    }

    #[derive(Debug, Default, serde::Serialize, serde::Deserialize, PartialEq)]
    struct TestStruct1 {
        foo: String,
        bar: i64,
        #[serde(serialize_with = "serialize_bigint", deserialize_with = "deserialize_bigint")]
        bigint: num_bigint::BigInt,
        ok: bool,
        nested_struct: TestStruct2,
        #[serde(deserialize_with="deserialize_byte_array")]
        bytearray: Vec<u8>,
    }

    let ts1 = TestStruct1 {
        foo: "foo".to_string(), bar: 1, ok: true,
        bigint: num_bigint::BigInt::from(170141183460469231731687303715884105727 as i128),
        nested_struct: TestStruct2{foo: "bar".to_string()}, bytearray: vec![1, 2, 3, 4, 5]
    };

    let r: Params = Params::from_struct(&ts1);
    let m: Result<TestStruct1, String> = r.to_struct();

    assert_eq!(ts1, m.unwrap());
    
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
    params.insert("m".to_string(), Params::ByteArray(bytearray_value.to_vec()));
    params.insert("array".to_string(), Params::Array(vec![Params::Integer(1), Params::Text("foo".to_string())]));

    let params_dict = Params::Dict(params);
    let result: Result<TestStruct, String> = params_dict.to_struct();

    if let Ok(val) = result {
        assert_eq!(ts, val);
    } else {
        panic!("Error deserializing params: {}", result.unwrap_err());
    }
}