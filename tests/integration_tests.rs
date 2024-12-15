use postchain_client::{
    transport::rest_client::{self, RestClient, RestResponse}, utils::params::{Params, QueryParams}
};

use std::{collections::BTreeMap, str::FromStr};

const POSTCHAIN_SINGLE_NODE_API_URL: &str = "http://localhost:7740";
const POSTCHAIN_MULTI_NODE_API_URL: &str = "https://node0.devnet1.chromia.dev:7740";

async fn assert_roundtrips<'a>(
    rc: &RestClient<'_>,
    brid: &str,
    query_type: &str,
    query_args: Option<&'a mut Vec<(&str, Params<'_>)>>,
    expected_value: &str,
) {
    let do_query = rc.query(&brid, None, query_type, None, query_args).await;

    print!("test query = {} ... ", query_type);

    match do_query {
        Ok(val) => {
            if let RestResponse::Bytes(val1) = val {
                assert_eq!(hex::encode(val1), expected_value);
                println!("ok")
            }
        }
        Err(error) => {
            rc.print_error(&error);
            std::process::exit(0);
        }
    }
}

#[allow(unused_assignments)]
#[tokio::test]
async fn queries_integration_test_success_cases() {
    let mut rc = rest_client::RestClient {
        node_url: vec![POSTCHAIN_SINGLE_NODE_API_URL],
        ..Default::default()
    };

    let mut url = String::new();
    
    let get_blockchain_rid = rc.get_blockchain_rid(0).await;

    let brid_info: (String, &RestClient) = if let Ok(val) = get_blockchain_rid {
        (val, &rc)
    } else {
        let brid = "7A37DD331AC8FED64EEFCCA231B0F975DE7F4371CE5CA44105A5B117DF6DE251".to_string();

        rc = rest_client::RestClient {
            node_url: vec![POSTCHAIN_MULTI_NODE_API_URL],
            ..Default::default()
        };

        let result = rc.get_nodes_from_directory(
            &brid,
        ).await;

        if let Err(error) = result {
            rc.print_error(&error);
            std::process::exit(0);
        }

        url = result.unwrap()[0].clone();
    
        rc = rest_client::RestClient {
            node_url: vec![&url],
            ..Default::default()
        };

        (brid, &rc)
    };

    let brid = brid_info.0;
    let rc = brid_info.1;   

    // query boolean
    assert_roundtrips(
        &rc,
        &brid,
        "test_boolean",
        Some(&mut vec![("arg1", QueryParams::Boolean(false))]),
        "a303020101",
    )
    .await;

    // query number
    assert_roundtrips(
        &rc,
        &brid,
        "test_number",
        Some(&mut vec![("arg1", QueryParams::Integer(1000))]),
        "a304020203e8",
    )
    .await;

    // query negative number
    assert_roundtrips(
        &rc,
        &brid,
        "test_number",
        Some(&mut vec![("arg1", QueryParams::Integer(-1000))]),
        "a3040202fc18",
    )
    .await;

    // query decimal
    assert_roundtrips(
        &rc,
        &brid,
        "test_decimal",
        Some(&mut vec![("arg1", QueryParams::Decimal(99.999))]),
        "a2080c0639392e393939",
    )
    .await;

    // query string
    assert_roundtrips(
        &rc,
        &brid,
        "test_string",
        Some(&mut vec![("arg1", QueryParams::Text("test"))]),
        "a2060c0474657374",
    )
    .await;

    // query byteArray
    assert_roundtrips(
        &rc,
        &brid,
        "test_byte_array",
        Some(&mut vec![(
            "arg1",
            QueryParams::ByteArray("test".as_bytes()),
        )]),
        "a106040474657374",
    )
    .await;

    // query json
    let data = serde_json::json!({
        "name": "Cuong Le",
        "city": "HCM",
        "country": "Vietnam"
    })
    .to_string();

    assert_roundtrips(&rc, &brid, "test_json", Some(&mut vec![
        ("arg1", QueryParams::Text(&data))
    ]), "a2360c347b2263697479223a2248434d222c22636f756e747279223a22566965746e616d222c226e616d65223a2243756f6e67204c65227d").await;

    // query null
    assert_roundtrips(&rc, &brid, "test_null", None, "a0020500").await;

    // query big integer
    let data = num_bigint::BigInt::from_str("1234567890123456789").unwrap();
    assert_roundtrips(
        &rc,
        &brid,
        "test_big_integer",
        Some(&mut vec![("arg1", QueryParams::BigInteger(data))]),
        "a60a0208112210f47de98115",
    )
    .await;

    // query array
    let data = &mut vec![(
        "arg1",
        QueryParams::Array(vec![
            QueryParams::Text("foo"),
            QueryParams::Text("bar"),
        ]),
    )];
    assert_roundtrips(
        &rc,
        &brid,
        "test_array",
        Some(data),
        "a510300ea2050c03666f6fa2050c03626172",
    )
    .await;

    // query empty array
    let data = &mut vec![("arg1", QueryParams::Array(vec![]))];
    assert_roundtrips(&rc, &brid, "test_array", Some(data), "a5023000").await;

    // query string key map
    let mut params: BTreeMap<&str, QueryParams> = BTreeMap::new();
    params.insert("foo", QueryParams::Text("bar"));
    params.insert("foo1", QueryParams::Text("bar1"));

    let data = &mut vec![("arg1", QueryParams::Dict(params))];

    assert_roundtrips(
        &rc,
        &brid,
        "test_string_key_map",
        Some(data),
        "a420301e300c0c03666f6fa2050c03626172300e0c04666f6f31a2060c0462617231",
    )
    .await;

    // query empty string key map
    let params: BTreeMap<&str, QueryParams> = BTreeMap::new();
    let data = &mut vec![("arg1", QueryParams::Dict(params))];
    assert_roundtrips(&rc, &brid, "test_string_key_map", Some(data), "a4023000").await;

    // query set
    let data = &mut vec![(
        "arg1",
        QueryParams::Array(vec![
            QueryParams::Text("foo"),
            QueryParams::Text("bar"),
            QueryParams::Text("foo1"),
            QueryParams::Text("bar1"),
        ]),
    )];
    assert_roundtrips(
        &rc,
        &brid,
        "test_set",
        Some(data),
        "a520301ea2050c03666f6fa2050c03626172a2060c04666f6f31a2060c0462617231",
    )
    .await;

    // query empty set
    // see: `query empty array`

    // query unnamed tuple
    // same `array`
    let data = &mut vec![(
        "arg1",
        QueryParams::Array(vec![QueryParams::Integer(1), QueryParams::Integer(2)]),
    )];
    assert_roundtrips(
        &rc,
        &brid,
        "test_unnamed_tuple",
        Some(data),
        "a50c300aa303020101a303020102",
    )
    .await;

    // query named tuple
    // same `map`
    let mut params: BTreeMap<&str, QueryParams> = BTreeMap::new();
    params.insert("x", QueryParams::Integer(1));
    params.insert("y", QueryParams::Integer(2));

    let data = &mut vec![("arg1", QueryParams::Dict(params))];
    assert_roundtrips(
        &rc,
        &brid,
        "test_named_tuple",
        Some(data),
        "a416301430080c0178a30302010130080c0179a303020102",
    )
    .await;

    // query empty string key map
    // see `query empty string key map`

    // query set
    let data = &mut vec![(
        "arg1",
        QueryParams::Array(vec![
            QueryParams::Text("foo"),
            QueryParams::Text("bar"),
        ]),
    )];
    assert_roundtrips(
        &rc,
        &brid,
        "test_set",
        Some(data),
        "a510300ea2050c03666f6fa2050c03626172",
    )
    .await;

    // query empty set
    // same `array`

    // queries tuple test
    // same `array`

    // query enum
    let data = &mut vec![("x", QueryParams::Integer(1))];
    assert_roundtrips(&rc, &brid, "test_enum", Some(data), "a303020101").await;

    // query struct
    // key = string
    // value = dict() or array()
    let mut params: BTreeMap<&str, QueryParams> = BTreeMap::new();
    params.insert("int", QueryParams::Integer(13));
    let data = &mut vec![("x", QueryParams::Dict(params))];
    assert_roundtrips(
        &rc,
        &brid,
        "test_struct",
        Some(data),
        "a40e300c300a0c03696e74a30302010d",
    )
    .await;

    let data = &mut vec![(
        "x",
        QueryParams::Array(vec![QueryParams::Integer(13)]),
    )];
    assert_roundtrips(
        &rc,
        &brid,
        "test_struct",
        Some(data),
        "a40e300c300a0c03696e74a30302010d",
    )
    .await;

    // query test map
    let data: &mut Vec<(&str, QueryParams)> = &mut vec![];

    assert_roundtrips(
        &rc,
        &brid,
        "test_map",
        Some(data),
        "a420301e301c0c0a73616d706c655f6b6579a20e0c0c73616d706c655f76616c7565",
    )
    .await;

    // query test map with bytearray key
    let data: &mut Vec<(&str, QueryParams)> = &mut vec![];

    assert_roundtrips(
        &rc,
        &brid,
        "test_map_with_bytearray_key",
        Some(data),
        "a53b3039a5373035a12304210373599a61cc6b3bc02a78c34313e1737ae9cfd56b9bb24360b437d469efdf3b15a20e0c0c73616d706c655f76616c7565",
    )
    .await;

    // query test nullable struct
    let mut params: BTreeMap<&str, QueryParams> = BTreeMap::new();
    params.insert("int", QueryParams::Null);

    let data = &mut vec![("arg1", QueryParams::Dict(params))];

    assert_roundtrips(
        &rc,
        &brid,
        "test_nullable_struct",
        Some(data),
        "a40d300b30090c03696e74a0020500",
    )
    .await;    

    // query test type as arg name
    assert_roundtrips(
        &rc,
        &brid,
        "test_type_as_arg_name",
        Some(&mut vec![("type", QueryParams::Text("test"))]),
        "a2060c0474657374",
    )
    .await;

    // query test complex object
    let client_data = serde_json::json!({
        "type": "data",
        "from": "client",
        "data": {
            "foo": "bar",
            "is_client": true
        }
    }).to_string();

    let server_data = serde_json::json!({
        "type": "data",
        "from": "server",
        "data": {
            "foo": "bar",
            "is_client": false
        }
    }).to_string();

    let blessing_rating_factor: BTreeMap<&str, QueryParams> = BTreeMap::new();
    let item_rating_factor: BTreeMap<&str, QueryParams> = BTreeMap::new();

    let mut args: BTreeMap<&str, QueryParams> = BTreeMap::new();
    args.insert("skill_unlock_level", QueryParams::Array(vec![
        QueryParams::Integer(1), QueryParams::Integer(2)
        ]));
    args.insert("hero_level_lookup", QueryParams::Array(vec![]));
    args.insert("player_level_lookup", QueryParams::Array(vec![]));
    args.insert("hero_level_bonus_lookup", QueryParams::Array(vec![]));
    args.insert("blessing_rating_factor", QueryParams::Dict(blessing_rating_factor));
    args.insert("item_rating_factor", QueryParams::Dict(item_rating_factor));
    args.insert("blessing_gender_male_chance", QueryParams::Decimal(1.1));
    args.insert("onboarding_map_blessing_to_fragments", QueryParams::Array(vec![]));
    args.insert("season_claim_offset", QueryParams::Integer(1));

    assert_roundtrips(
        &rc,
        &brid,
        "test_complex_object",
        Some(&mut vec![
            ("client_data", QueryParams::Text(&client_data)),
            ("server_data", QueryParams::Text(&server_data)),
            ("args", QueryParams::Dict(args))
        ]),
        "a2020c00",
    )
    .await;
}

#[tokio::test]
async fn queries_integration_test_get_nodes_from_directory() {
    let mut rc = rest_client::RestClient {
        node_url: vec![POSTCHAIN_MULTI_NODE_API_URL],
        ..Default::default()
    };

    let result = rc
        .get_nodes_from_directory(
            "4F2F41730E4CACBCA0A43F07AB756DCF57B8D72F4C1006825106D7B3C22758B0",
        )
        .await;

    let expected_result = vec![
        "https://node4.devnet1.chromia.dev:7740",
        "https://node7.devnet1.chromia.dev:7740",
        "https://node5.devnet1.chromia.dev:7740",
        "https://node6.devnet1.chromia.dev:7740",
    ];

    match result {
        Ok(val) => {
            rc.update_node_urls(&val);
            assert_eq!(rc.node_url, expected_result);
        }
        Err(error) => {
            rc.print_error(&error);
            std::process::exit(0);
        }
    }
}
