# Postchain Client Rust

A Rust client library for interacting with the Chromia blockchain deployed to a Postchain single node (manual mode) or multi-nodes managed by Directory Chain (managed mode).

This library provides functionality for executing queries, creating and signing transactions, and managing blockchain operations.

## Installation

Add this to your `Cargo.toml`:

#### For only use the `postchain_client::utils::operation::Params` enum to construct data for queries and transactions.

```toml
[dependencies]
postchain-client = "0.0.1"
tokio = { version = "1.42.0", features = ["rt"] }
```

#### For the both use the `postchain_client::utils::operation::Params` enum and the Rust's struct to serialize and deserialize with `serde`:

```toml
[dependencies]
postchain-client = "0.0.1"
tokio = { version = "1.42.0", features = ["rt"] }
serde = { version = "1.0.216", features = ["derive"] }
serde_json = { version = "1.0", features = ["preserve_order"] }
```

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

Queries allow you to fetch data from the blockchain:

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

    let query_arguments = Params::from_struct_to_vec(&QueryArguments {
        arg1: "value1".to_string(), arg2: "value2".to_string()
    });

    let mut query_arguments_ref: Vec<(&str, Params)> = query_arguments.iter().map(|v| (v.0.as_str(), v.1.clone())).collect();

    let result = client.query(
        "<BLOCKCHAIN_RID>",
        None,
        query_type,
        None,
        Some(&mut query_arguments_ref)
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

#### 3.4 Error Handling

The library uses Rust's Result type for error handling:

```rust
match client.send_transaction(&tx).await {
    Ok(resp) => {
        println!("Transaction sent successfully: {:?}", resp);
    },
    Err(err) => {
        eprintln!("Error sending transaction: {:?}", err);
    }
}
```

### 5. Parameter Types

The library supports various parameter types through the `Params` enum and Rust struct :

| GTV(*)  | Rust types | Params enums | ASN.1 DER |
| --- | --- | --- | --- |
| null | `Option<T> = None` | Params::Null | NULL |
| integer | bool | Params::Boolean(bool) | INTEGER |
| integer | i64 | Params::Integer(i64) | INTEGER |
| bigInteger | i128 | Params::BigInteger(num_bigint::BigIn) | INTEGER |
| decimal | f64 | Params::Decimal(f64) | UTF8String |
| string | String | Params::Text(String) | UTF8String |
| array | `Vec<T>` | Params::Array(`Vec<Params>`) | SEQUENCE |
| dict | `BTreeMap<K, V>` | Params::Dict(`BTreeMap<String, Params>`) | SEQUENCE |
| byteArray | `Vec<u8>` | Params:: ByteArray(`Vec<u8>`) | OCTET STRING |

(*) GTV gets converted to ASN.1 DER when it's sent. See more : https://docs.chromia.com/intro/architecture/generic-transaction-protocol#generic-transfer-value-gtv

We don't currently support this Rust type: `&str` string slice type for string parameters.

## Examples

### Book Review Application Example

Here's a real-world example from a book review application that demonstrates querying, creating transactions, and handling structured data: https://github.com/cuonglb/postchain-client-rust/tree/dev/examples/book-review

This example demonstrates:
- Defining and using structured data with serde
- Querying the blockchain and handling responses
- Creating and sending transactions
- Signing transactions with a private key
- Error handling and status checking

### How To Run

Start a Postchain single node with the `book-review` Rell dapp:
```shell
$ cd examples/book-review/rell-dapp/
$ sudo docker compose up -d
```

Start a simple Rust application to interact with thebook-review blockchain:
```shell
$ cd examples/book-review/client
$ cargo run
```

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the terms specified in the [LICENSE](LICENSE) file.
