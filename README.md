# Postchain Client Rust

A Rust client library for interacting with the [Chromia](https://chromia.com/) blockchain deployed to a Postchain single node (manual mode) or multi-nodes managed by Directory Chain (managed mode).

This library provides functionality for executing queries, creating and signing transactions, and managing blockchain operations.

## Installation

Add this to your `Cargo.toml`:

#### For only use the `postchain_client::utils::operation::Params` enum to construct data for queries and transactions.

```toml
[dependencies]
postchain-client = "0.0.2"
tokio = { version = "1.42.0", features = ["rt"] }
```

#### For the both use the `postchain_client::utils::operation::Params` enum and the Rust's struct to serialize and deserialize with `serde`:

```toml
[dependencies]
postchain-client = "0.0.2"
tokio = { version = "1.42.0", features = ["rt"] }
serde = { version = "1.0.216", features = ["derive"] }
serde_json = { version = "1.0", features = ["preserve_order"] }
```

## Documentation for `postchain_client` latest crate

https://docs.rs/postchain-client/latest/postchain_client/

## Usage Guide

### 1. Setting Up the Client

```rust
use postchain_client::transport::client::RestClient;

let client = RestClient {
    node_url: vec!["http://localhost:7740", "http://localhost:7741"],
    request_time_out: 30,
    poll_attemps: 5,
    poll_attemp_interval_time: 5
};
```

### 2. Executing Queries

Queries allow our to fetch data from the blockchain:

```rust
use postchain_client::utils::operation::Params;

async fn execute_query_with_params(client: &RestClient<'_>) -> Result<(), Box<dyn std::error::Error>> {
    let query_type = "<query_name>";
    let mut query_arguments = vec![
        ("arg1", Params::Text("value1".to_string())),
        ("arg2", Params::Text("value2".to_string())),
    ];
    
    let result = client.query(
        "<BLOCKCHAIN_RID>",
        None,
        query_type,
        None,
        Some(&mut query_arguments)
    ).await?;

    Ok(())
}

async fn execute_query_with_struct(client: &RestClient<'_>) -> Result<(), Box<dyn std::error::Error>> {
    let query_type = "<query_name>";

    #[derive(Debug, Default, serde::Serialize)]
    struct QueryArguments {
        arg1: String,
        arg2: String
    }

    let mut query_arguments = Params::from_struct_to_vec(&QueryArguments {
        arg1: "value1".to_string(), arg2: "value2".to_string()
    });

    let result = client.query(
        "<BLOCKCHAIN_RID>",
        None,
        query_type,
        None,
        Some(&mut query_arguments)
    ).await?;

    if let RestResponse::Bytes(val1) = result {
        println!("{:?}", gtv::decode(&val1));
    }

    Ok(())
}
```


### 3. Creating and Sending Transactions

#### 3.1 Creating Operations

```rust
use postchain_client::utils::operation::{Operation, Params};

// Create operation with named parameters (dictionary)
let operation = Operation::from_dict(
    "operation_name",
    vec![
        ("param1", Params::Text("value1".to_string())),
        ("param2", Params::Integer(42)),
    ]
);

// Or create operation with unnamed parameters (list)
let operation = Operation::from_list(
    "operation_name",
    vec![
        Params::Text("value1".to_string()),
        Params::Integer(42),
    ]
);
```

#### 3.2 Creating and Signing Transactions

```rust
use postchain_client::utils::transaction::Transaction;

// Create new transaction
let mut tx = Transaction::new(
    brid_hex_decoded.to_vec(),    // blockchain RID in hex decode to vec
    Some(vec![operation]),      // operations
    None,                       // signers (optional)
    None                        // signatures (optional)
);

// Sign transaction with private key
let private_key = [0u8; 64];  // Your private key bytes
tx.sign(&private_key)?;

// Or sign with multiple private keys
let private_keys = vec![&private_key1, &private_key2];
tx.multi_sign(&private_keys)?;
```

#### 3.3 Sending Transactions

```rust
async fn send_transaction(client: &RestClient<'_>, tx: &Transaction<'_>) -> Result<(), Box<dyn std::error::Error>> {
    // Send transaction
    let response = client.send_transaction(tx).await?;
    
    // Get transaction RID (for status checking)
    let tx_rid = tx.tx_rid_hex();
    
    // Check transaction status
    let status = client.get_transaction_status("<blockchain RID>", &tx_rid).await?;
    
    Ok(())
}
```

### 4. Error and Response Handling

The response from `client.query` and `client.send_transaction` is a `postchain_client::transport::client::RestResponse` enum if success
or a `postchain_client::transport::client::RestError` enum if failed.

We can handle it as follows:

```rust
let result = client.query(/* ... */).await;

match result {
    Ok(resp: RestResponse) => {
        if let RestResponse::Bytes(val1) = resp {
            let params = gtv::decode(&val1);
            /// Do whatever we want with the decoded params
        }
    },
    Error(error: RestError) => {
        /// Do whatever we want with the error
    }
}
```

```rust
let result = client.send_transaction(&tx).await;

match result {
    Ok(resp: RestResponse) => {
        println!("Transaction sent successfully: {:?}", resp);
    },
    Err(error: ) => {
        eprintln!("Error sending transaction: {:?}", err);
    }
}
```

The response from `client.get_transaction_status` is a `postchain_client::utils::transaction::TransactionStatus` enum if success or a `postchain_client::transport::client::RestError` enum if failed.

We can handle it as follows:

```rust
let result = client.get_transaction_status("<blockchain RID>", &tx_rid).await;

match result {
    Ok(resp: TransactionStatus) => {
        /// Do anything else here
    },
    Err(error: RestError) => {
        /// Do whatever we want with the error
    }
}
```

### 5. Use `serde` for `serialize` or `deserialize` a struct to `Params::Dict` or vice versa

Please look at all tests in `operation.rs` source here: https://github.com/cuonglb/postchain-client-rust/blob/dev/src/utils/operation.rs

### 6. Logging

`postchain-client` uses `tracing` crate for logging. You can use `tracing-subscriber` crate to enable all logs.

```rust
use tracing_subscriber;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    ...
}
```

### 7. Parameter Types

The library supports various parameter types through the `Params` enum and Rust struct :

| GTV(*)  | Rust types or 3rd crates | Postchain Client Params enums | Note |
| --- | --- | --- | --- |
| null | `Option<T> = None` | Params::Null | |
| integer | bool | Params::Boolean(bool) | |
| integer | i64 | Params::Integer(i64) | |
| bigInteger | num_bigint::BigInt | Params::BigInteger(num_bigint::BigInt) | (**) |
| decimal | bigdecimal::BigDecimal | Params::Decimal(bigdecimal::BigDecimal) | (***) |
| string | String | Params::Text(String) | |
| array | `Vec<T>` | Params::Array(`Vec<Params>`) | |
| dict | `BTreeMap<K, V>` | Params::Dict(`BTreeMap<String, Params>`) | |
| byteArray | `Vec<u8>` | Params:: ByteArray(`Vec<u8>`) | |


(*) GTV gets converted to ASN.1 DER when it's sent. See more : https://docs.chromia.com/intro/architecture/generic-transaction-protocol#generic-transfer-value-gtv

(**) We can use serde custom derive macros in some of cases to:

> Handle arbitrary-precision integers:
- `operation::deserialize_bigint` for deserialization.
- `operation::serialize_bigint` for serialization.
```rust
use postchain_client::utils::{operation::{deserialize_bigint, serialize_bigint}};
...
    #[derive(serde::Serialize, serde::Deserialize)]
    struct TestStructBigInt {
        #[serde(serialize_with = "serialize_bigint", deserialize_with = "deserialize_bigint")]
        bigint: num_bigint::BigInt
    }
...
```

> Handle arbitrary-precision decimal:
- `operation::deserialize_bigint` for deserialization.
- `operation::serialize_bigint` for serialization.
```rust
use postchain_client::utils::{operation::{serialize_bigdecimal, deserialize_bigdecimal}};
...
    #[derive(serde::Serialize, serde::Deserialize)]
    struct TestStructDecimal {
        #[serde(serialize_with = "serialize_bigdecimal", deserialize_with = "deserialize_bigdecimal")]
        bigdecimal: bigdecimal::BigDecimal
    }
...
```

## Examples

### Book Review Application Example

Here's a real-world example from a book review application that demonstrates querying, creating transactions, and handling structured data: https://github.com/cuonglb/postchain-client-rust/tree/dev/examples/book-review

This example demonstrates:
- Defining and using structured data with serde
- Querying the blockchain and handling responses
- Creating and sending transactions
- Signing transactions with a private key
- Transaction error handling and status checking

### How To Run

Install Rust (https://www.rust-lang.org/tools/install)
and Docker with compose.

Start a Postchain single node with the `book-review` Rell dapp:
```shell
$ cd examples/book-review/rell-dapp/
$ sudo docker compose up -d
```

Start a simple Rust application to interact with the book-review blockchain:
```shell
$ cd examples/book-review/client
$ cargo run
```

### Other

https://github.com/cuonglb/postchain-client-rust/tree/dev/examples/for-docs

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the terms specified in the [LICENSE](LICENSE) file.
