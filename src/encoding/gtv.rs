use crate::utils::{params::Params, transaction::Transaction};

use asn1::{Asn1Read, Asn1Readable, Asn1Write, ParseError};
use std::collections::BTreeMap;

#[derive(Asn1Read, Asn1Write, Debug, Clone)]
pub enum Choice<'a> {
    #[explicit(0)]
    NULL(()),
    #[explicit(1)]
    OCTETSTRING(&'a [u8]),
    #[explicit(2)]
    UTF8STRING(asn1::Utf8String<'a>),
    #[explicit(3)]
    INTEGER(i64),
    #[explicit(4)]
    DICT(asn1::Sequence<'a>),
    #[explicit(5)]
    ARRAY(asn1::Sequence<'a>),
    #[explicit(6)]
    BIGINTEGER(asn1::BigInt<'a>),
}

pub trait GTVParams<'a>: Clone {
    fn to_writer(&self, writer: &mut asn1::Writer) -> asn1::WriteResult;
}

#[allow(unused_assignments)]
impl<'a> GTVParams<'a> for Params<'a> {
    fn to_writer(&self, writer: &mut asn1::Writer) -> asn1::WriteResult {
        if let Params::Array(val) = self {
            writer.write_explicit_element(
                &asn1::SequenceWriter::new(&|writer: &mut asn1::Writer| {
                    for v in val {
                        v.to_writer(writer)?;
                    }
                    Ok(())
                }),
                5,
            )?;
            Ok(())
        } else if let Params::Dict(val) = self {
            writer.write_explicit_element(
                &asn1::SequenceWriter::new(&|writer: &mut asn1::Writer| {
                    for v in val {
                        writer.write_element(&asn1::SequenceWriter::new(
                            &|writer: &mut asn1::Writer| {
                                writer.write_element(&asn1::Utf8String::new(v.0))?;
                                v.1.to_writer(writer)?;
                                Ok(())
                            },
                        ))?;
                    }

                    Ok(())
                }),
                4,
            )?;
            Ok(())
        } else {
            let mut decimal_to_string = String::new();
            let mut bigint_to_vec_u8: Vec<u8> = Vec::new();

            let gtv_choice = match self {
                Params::Integer(val) => Choice::INTEGER(*val),
                Params::Boolean(val) => Choice::INTEGER(*val as i64),
                Params::Decimal(val) => {
                    decimal_to_string = val.to_string();
                    Choice::UTF8STRING(asn1::Utf8String::new(decimal_to_string.as_str()))
                }
                Params::Text(val) => Choice::UTF8STRING(asn1::Utf8String::new(val)),
                Params::ByteArray(val) => Choice::OCTETSTRING(&val),
                Params::BigInteger(val) => {
                    bigint_to_vec_u8 = val.to_bytes_be().1;
                    Choice::BIGINTEGER(asn1::BigInt::new(bigint_to_vec_u8.as_slice()).unwrap())
                }
                _ => Choice::NULL(())
            };

            writer.write_element(&gtv_choice)?;
            Ok(())
        }
    }
}

pub fn encode_tx<'a>(tx: &Transaction<'a>) -> Vec<u8> {
  asn1::write(|writer| {
    writer.write_explicit_element(
      &asn1::SequenceWriter::new(&|writer: &mut asn1::Writer| {
          
          writer.write_explicit_element(
            &asn1::SequenceWriter::new(&|writer: &mut asn1::Writer| {

              // Blockchain RID
              writer.write_element(&Choice::OCTETSTRING(
                &hex::decode(tx.blockchain_rid).unwrap()))?;

              // Operations and args
              writer.write_explicit_element(
                &asn1::SequenceWriter::new(&|writer: &mut asn1::Writer| {
 
                  encode_tx_body(writer, &Some(&mut tx.operations.clone()))?;      

                  Ok(())
              }), 5)?;


              // Signers pubkeys
              writer.write_explicit_element(
                &asn1::SequenceWriter::new(&|writer: &mut asn1::Writer| {
                
                  for sig in &tx.signers {
                    writer.write_element(&Choice::OCTETSTRING(&sig))?;
                  }

                  Ok(())
              }), 5)?;

              Ok(())
          }), 5)?;

          // Signatures
          writer.write_explicit_element(
            &asn1::SequenceWriter::new(&|writer: &mut asn1::Writer| {
             
              for sig in &tx.signatures {
                writer.write_element(&Choice::OCTETSTRING(&sig))?;
              }

              Ok(())
          }), 5)?;

        Ok(())
      }),
      5, )?;
    Ok(())
  }).unwrap()
}

pub fn encode<'a>(
    query_type: &str,
    query_args: Option<&'a mut Vec<(&str, Params<'_>)>>,
) -> Vec<u8> {
    asn1::write(|writer| {
        writer.write_explicit_element(
            &asn1::SequenceWriter::new(&|writer: &mut asn1::Writer| {
                writer.write_element(&Choice::UTF8STRING(asn1::Utf8String::new(query_type)))?;
                encode_body(writer, &query_args)?;
                Ok(())
            }),
            5,
        )?;
        Ok(())
    })
    .unwrap()
}

fn encode_tx_body<'a>(writer: &mut asn1::Writer,
  query_args: &Option<&'a mut Vec<(&str, Params<'_>)>>)
  -> asn1::WriteResult {
    if let Some(q_args) = &query_args {
      let q_args_as_slice = q_args.iter().as_slice();
      for (q_type, q_args) in q_args_as_slice {
      writer.write_explicit_element(
          &asn1::SequenceWriter::new(&|writer: &mut asn1::Writer| {

            // Operation name
            writer.write_explicit_element(&asn1::Utf8String::new(&q_type), 2)?;

            // Operation args
            writer.write_explicit_element(
          &asn1::SequenceWriter::new(&|writer: &mut asn1::Writer| {
              q_args.to_writer(writer)?;
              Ok(())
            }),5)?;        
            
            Ok(())
          }),
          5,
      )?;
    }
  }
  Ok(())
}

fn encode_body<'a>(writer: &mut asn1::Writer,
  query_args: &Option<&'a mut Vec<(&str, Params<'_>)>>)
  -> asn1::WriteResult {
  writer.write_explicit_element(
      &asn1::SequenceWriter::new(&|writer: &mut asn1::Writer| {
          if let Some(q_args) = &query_args {
              let q_args_as_slice = q_args.iter().as_slice();
              for (q_type, q_args) in q_args_as_slice {
                  writer.write_element(&asn1::SequenceWriter::new(
                      &|writer: &mut asn1::Writer| {
                          writer.write_element(&asn1::Utf8String::new(&q_type))?;
                          q_args.to_writer(writer)?;
                          Ok(())
                      },
                  ))?;
              }
          }
          Ok(())
      }),
      4,
  )?;
  Ok(())
}

fn decode_simple(choice: Choice) -> Params {
  match choice {
      Choice::INTEGER(val) =>
        Params::Integer(val),
      Choice::BIGINTEGER(val) => {
        Params::BigInteger(
          num_bigint::BigInt::from_bytes_be(
            if val.is_negative() { num_bigint::Sign::Minus } else { num_bigint::Sign::Plus }, 
            val.as_bytes().try_into().unwrap()))
      },
      Choice::OCTETSTRING(val) =>
        Params::ByteArray(val),
      Choice::UTF8STRING(val) =>
        Params::Text(val.as_str()),
      _ => 
        Params::Null
  }
}

fn decode_sequence_array<'a>(parser: &mut asn1::Parser<'a>, vec_array: &mut Vec<Params<'a>>) {
  while let Ok(val) = Choice::parse(parser) {
    let op_val = match val {
        Choice::ARRAY(seq) => {
          let res: Result<Params, ParseError> = seq.parse(|parser| {
            let mut vect_array_new: Vec<Params> = Vec::new();
            decode_sequence_array(parser, &mut vect_array_new);
            Ok(Params::Array(vect_array_new))
          });
          res.unwrap()
        }
        Choice::DICT(seq) => {
          let res: Result<Params, ParseError> = seq.parse(|parser| {
            let mut btree_map_new: BTreeMap<String, Params> = BTreeMap::new();
            decode_sequence_dict(parser, &mut btree_map_new);
            Ok(Params::Dict(btree_map_new))
          });
          res.unwrap()
        }
        _ =>
          decode_simple(val)
    };
    vec_array.push(op_val);
  }
}

fn decode_sequence_dict<'a>(parser: &mut asn1::Parser<'a>, btreemap: &mut BTreeMap<String, Params<'a>>) {
  loop {
      let seq = parser.read_element::<asn1::Sequence>();
      if let Err(_) = seq {
          break;
      }
      let res: Result<(String, Params), ParseError> = seq.unwrap().parse(|parser| {
        let key = parser.read_element::<asn1::Utf8String>()?;
        let val = Choice::parse(parser).unwrap();

        let op_val = match val {
          Choice::DICT(seq) => {
            let res: Result<Params, ParseError> = seq.parse(|parser| {
              let mut btree_map_new: BTreeMap<String, Params> = BTreeMap::new();
              decode_sequence_dict(parser, &mut btree_map_new);
              Ok(Params::Dict(btree_map_new))
            });
            res.unwrap()
          }
          Choice::ARRAY(seq) => {
            let res: Result<Params, ParseError> = seq.parse(|parser| {
              let mut vect_array_new: Vec<Params> = Vec::new();
              decode_sequence_array(parser, &mut vect_array_new);
              Ok(Params::Array(vect_array_new))
            });
            res.unwrap()
          },
          _ => 
            decode_simple(val)      
        };

        Ok((key.as_str().to_string(), op_val))
      });

      let res = res.unwrap();

      btreemap.insert(res.0, res.1);
  }
}

pub fn decode<'a>(data: &'a [u8]) -> Result<Params<'a>, ParseError> {
  let tag = asn1::Tag::from_bytes(data).unwrap();
  let tag_num = tag.0.as_u8().unwrap() & 0x1f;

  if vec![0, 1, 2, 3, 6].contains(&tag_num) {
    asn1::parse(data, |d| {
        let res_choice = Choice::parse(d);
        match res_choice {
            Ok(val) => Ok(decode_simple(val)),
            Err(error) => Err(error),
        }
    })
  } else {
    if tag_num == 4 {
      let result = asn1::parse_single::<asn1::Explicit<asn1::Sequence, 4>>(data).unwrap();
      result.into_inner().parse(|parser| {
        let mut btree_map_new: BTreeMap<String, Params> = BTreeMap::new();
        decode_sequence_dict(parser, &mut btree_map_new);
        Ok(Params::Dict(btree_map_new))
      })
    } else if tag_num == 5 {
      let result = asn1::parse_single::<asn1::Explicit<asn1::Sequence, 5>>(data).unwrap();
      result.into_inner().parse(|parser|{
        let mut vect_array_new: Vec<Params> = Vec::new();
        decode_sequence_array(parser, &mut vect_array_new);
        Ok(Params::Array(vect_array_new))
      })
    } else {
      Ok(Params::Null)
    }
  }
}

pub fn decode_tx<'a>(data: &'a [u8]) -> Result<Params<'a>, ParseError> {
  decode(data)
}

#[allow(dead_code)]
fn assert_roundtrips<'a>(
  query_args: Option<&'a mut Vec<(&str, Params<'_>)>>,
  expected_value: &str) {
    let result = asn1::write(|writer| {
      encode_body(writer, &query_args)?;
      Ok(())
    });
    assert_eq!(hex::encode(result.unwrap()), expected_value);
}

#[test]
fn gtv_test_sequence_with_empty() {
  assert_roundtrips(None, "a4023000");
}

#[test]
fn gtv_test_sequence_with_boolean() {
  assert_roundtrips(Some(&mut vec![("foo", Params::Boolean(true))]), 
  "a40e300c300a0c03666f6fa303020101");
}

#[test]
fn gtv_test_sequence_with_string() {
  assert_roundtrips(Some(&mut vec![("foo", Params::Text("bar"))]), 
  "a410300e300c0c03666f6fa2050c03626172");
}

#[test]
fn gtv_test_sequence_with_octet_string() {
  assert_roundtrips(Some(&mut vec![("foo", Params::ByteArray("bar".as_bytes()))]), 
  "a410300e300c0c03666f6fa1050403626172");
}

#[test]
fn gtv_test_sequence_with_number() {
  assert_roundtrips(Some(&mut vec![("foo", Params::Integer(9999))]), 
  "a40f300d300b0c03666f6fa3040202270f");
}

#[test]
fn gtv_test_sequence_with_negative_number() {
  assert_roundtrips(Some(&mut vec![("foo", Params::Integer(-9999))]), 
  "a40f300d300b0c03666f6fa3040202d8f1");
}

#[test]
fn gtv_test_sequence_with_decimal() {
  assert_roundtrips(Some(&mut vec![("foo", Params::Decimal(99.99))]), 
  "a4123010300e0c03666f6fa2070c0539392e3939");
}

#[test]
fn gtv_test_sequence_with_negative_decimal() {
  assert_roundtrips(Some(&mut vec![("foo", Params::Decimal(-99.99))]), 
  "a4133011300f0c03666f6fa2080c062d39392e3939");
}

#[test]
fn gtv_test_sequence_with_json() {
  let data = serde_json::json!({
            "foo": "bar",
            "bar": 9,
            "foo": 9.00
        }).to_string();
  assert_roundtrips(Some(&mut vec![("foo", Params::Text(&data))]), 
  "a420301e301c0c03666f6fa2150c137b22626172223a392c22666f6f223a392e307d");
}

#[test]
fn gtv_test_sequence_with_big_integer() {
  use std::str::FromStr;

  let max_i128: i128 = i128::MAX;
  let data = num_bigint::BigInt::from_str(max_i128.to_string().as_str()).unwrap();
  assert_roundtrips(Some(&mut vec![("foo", Params::BigInteger(data))]), 
  "a41d301b30190c03666f6fa61202107fffffffffffffffffffffffffffffff");
}

#[test]
fn gtv_test_sequence_with_negative_big_integer() {
  use std::str::FromStr;

  let min_i128: i128 = i128::MIN;
  let data = num_bigint::BigInt::from_str(min_i128.to_string().as_str()).unwrap();
  assert_roundtrips(Some(&mut vec![("foo", Params::BigInteger(data))]), 
  "a41d301b30190c03666f6fa612021080000000000000000000000000000000");
}

#[test]
fn gtv_test_sequence_with_array() {
  let data = &mut vec![(
      "foo",
      Params::Array(vec![
          Params::Text("bar1"),
          Params::Text("bar2"),
      ]),
  )];
  assert_roundtrips(Some(data), 
  "a41d301b30190c03666f6fa5123010a2060c0462617231a2060c0462617232");
}

#[test]
fn gtv_test_sequence_with_dict() {
  use std::collections::BTreeMap;

  let mut params: BTreeMap<String, Params> = BTreeMap::new();
  params.insert("foo".to_string(), Params::Text("bar"));
  params.insert("foo1".to_string(), Params::Text("bar1"));

  let data = &mut vec![("foo",  Params::Dict(params))];

  assert_roundtrips(Some(data), 
  "a42b302930270c03666f6fa420301e300c0c03666f6fa2050c03626172300e0c04666f6f31a2060c0462617231");
}

#[test]
fn gtv_test_sequence_with_nested_dict() {
  use std::collections::BTreeMap;

  let mut dict1: BTreeMap<String, Params> = BTreeMap::new();
  let mut dict2: BTreeMap<String, Params> = BTreeMap::new();
  let dict3: BTreeMap<String, Params> = BTreeMap::new();

  dict1.insert("dict1_foo".to_string(), Params::Text("dict1_bar"));

  dict2.insert("dict2_foo".to_string(), Params::Text("dict2_bar"));
  dict2.insert("dict2_foo1".to_string(), Params::Text("dict2_bar1"));
  
  dict2.insert("dict3_empty_data".to_string(), Params::Dict(dict3));

  dict1.insert("dict2_data".to_string(), Params::Dict(dict2));

  let data = &mut vec![("foo",  Params::Dict(dict1))];

  assert_roundtrips(Some(data), 
  "a481893081863081830c03666f6fa47c307a30180c0964696374315f666f6fa20b0c0964696374315f626172305e0c0a64696374325f64617461a450304e30180c0964696374325f666f6fa20b0c0964696374325f626172301a0c0a64696374325f666f6f31a20c0c0a64696374325f6261723130160c1064696374335f656d7074795f64617461a4023000");
}

#[test]
fn gtv_test_sequence_with_nested_dict_array() {
  use std::collections::BTreeMap;

  let mut dict1: BTreeMap<String, Params> = BTreeMap::new();
  let mut dict2: BTreeMap<String, Params> = BTreeMap::new();
  let mut dict3: BTreeMap<String, Params> = BTreeMap::new();

  dict2.insert("dict2_foo".to_string(), Params::Text("dict2_bar"));
  dict3.insert("dict3_foo".to_string(), Params::Text("dict3_bar"));

  let array1 = vec![
    Params::Dict(dict2), Params::Dict(dict3)];

  dict1.insert("array1".to_string(), Params::Array(array1));

  let data = &mut vec![("foo",  Params::Dict(dict1))];

  assert_roundtrips(Some(data), 
  "a457305530530c03666f6fa44c304a30480c06617272617931a53e303ca41c301a30180c0964696374325f666f6fa20b0c0964696374325f626172a41c301a30180c0964696374335f666f6fa20b0c0964696374335f626172");
}

#[allow(dead_code)]
fn assert_roundtrips_simple(op: Params, expected_value: &str) {
  let result = asn1::write(|writer| {
      op.to_writer(writer)?;
      Ok(())
    });
  assert_eq!(hex::encode(result.unwrap()), expected_value);
}

#[test]
fn gtv_test_simple_null() {
  assert_roundtrips_simple(Params::Null, "a0020500");
}

#[test]
fn gtv_test_simple_boolean() {
  assert_roundtrips_simple(Params::Boolean(true), "a303020101");
  assert_roundtrips_simple(Params::Boolean(false), "a303020100");
}

#[test]
fn gtv_test_simple_integer() {
  assert_roundtrips_simple(Params::Integer(99999), "a305020301869f");
}

#[test]
fn gtv_test_simple_big_integer() {
  assert_roundtrips_simple(Params::BigInteger(num_bigint::BigInt::from(1234567890123456789 as i128)), "a60a0208112210f47de98115");
}

#[test]
fn gtv_test_simple_decimal() {
  assert_roundtrips_simple(Params::Decimal(99.999), "a2080c0639392e393939");
}

#[test]
fn gtv_test_simple_string() {
  assert_roundtrips_simple(Params::Text("abcABC123"), "a20b0c09616263414243313233");
  assert_roundtrips_simple(Params::Text("utf-8 unicode Trái Tim Ngục Tù ...!@#$%^&*()"), "a2320c307574662d3820756e69636f6465205472c3a1692054696d204e67e1bba5632054c3b9202e2e2e21402324255e262a2829");
}

#[test]
fn gtv_test_simple_byte_array() {
  assert_roundtrips_simple(Params::ByteArray(b"123456abcedf"), "a10e040c313233343536616263656466");
}

#[test]
fn gtv_test_simple_array() {
  assert_roundtrips_simple(Params::Array(vec![
    Params::Text("foo"), Params::Integer(1)
  ]), "a50e300ca2050c03666f6fa303020101");
}

#[test]
fn gtv_test_simple_dict() {
  use std::collections::BTreeMap;
  let mut data: BTreeMap<String, Params> = BTreeMap::new();
  data.insert("foo".to_string(), Params::Text("bar"));
  assert_roundtrips_simple(Params::Dict(data), "a410300e300c0c03666f6fa2050c03626172");
}

#[allow(dead_code)]
fn assert_roundtrips_simple_decode(data: &str, expected_value: Params) {
  let hex_decode_data = hex::decode(data).unwrap();
  let result = decode(&hex_decode_data).unwrap();
  assert_eq!(result, expected_value);
}

#[test]
fn gtv_test_simple_null_decode() {
  assert_roundtrips_simple_decode("a0020500", Params::Null);
}

#[test]
fn gtv_test_simple_big_integer_decode() {
  assert_roundtrips_simple_decode("a60a0208112210f47de98115", 
    Params::BigInteger(num_bigint::BigInt::from(1234567890123456789 as i128)));
}

#[test]
fn gtv_test_simple_integer_decode() {
  assert_roundtrips_simple_decode("a305020301869f", Params::Integer(99999));
}

#[test]
fn gtv_test_simple_decimal_decode() {
  assert_roundtrips_simple_decode("a2080c0639392e393939", Params::Text("99.999"));
}

#[test]
fn gtv_test_simple_string_decode() {
  assert_roundtrips_simple_decode("a2320c307574662d3820756e69636f6465205472c3a1692054696d204e67e1bba5632054c3b9202e2e2e21402324255e262a2829",
    Params::Text("utf-8 unicode Trái Tim Ngục Tù ...!@#$%^&*()"))
}

#[test]
fn gtv_test_simple_bytearray_with_hex_decode() {
  assert_roundtrips_simple_decode("a53b3039a5373035a12304210373599a61cc6b3bc02a78c34313e1737ae9cfd56b9bb24360b437d469efdf3b15a20e0c0c73616d706c655f76616c7565",
  Params::Array(vec![
    Params::Array(vec![
      Params::ByteArray(&hex::decode("0373599A61CC6B3BC02A78C34313E1737AE9CFD56B9BB24360B437D469EFDF3B15").unwrap()),
      Params::Text("sample_value")
    ])
  ]))
}

#[test]
fn gtv_test_sequence_simple_array_decode() {
  let data = Params::Array(vec![
    Params::Text("foo"), Params::Integer(1),
    Params::Text("bar"), Params::Integer(2),
    Params::Array(vec![]),
    Params::Text("ca"), Params::Integer(3),
    Params::Array(vec![
      Params::Integer(1111),
      Params::Array(vec![
        Params::Integer(2222),
        Params::Integer(3333),
      ])
    ]),
  ]);

  let result = asn1::write(|writer| {
      data.to_writer(writer)?; Ok(()) }).unwrap();
  
  assert_eq!(data, decode(result.as_slice()).unwrap());
}

#[test]
fn gtv_test_sequence_simple_dict_decode() {
  let mut data_btreemap: BTreeMap<String, Params> = BTreeMap::new();

  data_btreemap.insert("foo".to_string(), Params::Text("bar"));
  data_btreemap.insert("status".to_string(), Params::ByteArray("OK".as_bytes()));

  let data = Params::Dict(data_btreemap);

  let result = asn1::write(|writer| {
    data.to_writer(writer)?; Ok(()) }).unwrap();

  assert_eq!(data, decode(result.as_slice()).unwrap());  
}

#[test]
fn gtv_test_sequence_complex_mix_dict_array_decode() {
  use std::collections::BTreeMap;
  let mut data_btreemap: BTreeMap<String, Params> = BTreeMap::new();
  let mut dict_in: BTreeMap<String, Params> = BTreeMap::new();

  dict_in.insert("foo".to_string(), Params::Text("bar"));

  data_btreemap.insert("status".to_string(), Params::Text("dict_bar"));
  data_btreemap.insert("command".to_string(), Params::Text("dict_bar2"));
  data_btreemap.insert("state".to_string(), Params::Integer(123));
  data_btreemap.insert("dict".to_string(), Params::Dict(dict_in));
  data_btreemap.insert("array".to_string(), Params::Array(vec![
    Params::Text("test array"),
    Params::BigInteger(num_bigint::BigInt::from(123456 as i128)),
    Params::Array(vec![
      Params::Text("test array 2")
    ])
  ]));
  
  let data = Params::Dict(data_btreemap);

  let result = asn1::write(|writer| {
    data.to_writer(writer)?; Ok(()) }).unwrap();

  assert_eq!(data, decode(result.as_slice()).unwrap());
}