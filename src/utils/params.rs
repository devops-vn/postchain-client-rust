extern crate num_bigint;

use std::{collections::BTreeMap, fmt::Debug};

use num_bigint::BigInt;

#[derive(Clone, Debug, PartialEq)]
pub enum Params<'a> {
    Null,
    Boolean(bool),
    Integer(i64),
    BigInteger(BigInt),
    Decimal(f64),
    Text(&'a str),
    ByteArray(&'a [u8]),
    Array(Vec<Params<'a>>),
    Dict(BTreeMap<String, Params<'a>>),
}

pub type QueryParams<'a> = Params<'a>;
pub type OperationParams<'a> = Params<'a>;

#[derive(Debug)]
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
