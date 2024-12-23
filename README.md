#### Chromia Postchain Client for Rust ####

##### [Postchain Client is a set of predefined functions and utilities offering a convenient and simplified interface for interacting with a decentralized application (dapp) built using the Postchain blockchain framework, also known as Chromia.]

##### [This project is still under development; all feedback and contributions are welcome. Thanks!]

##### [Compile and run tests from source] #####

###### Note: This is specifically for *nix operating systems, but We haven't tested it on BSD or Windows yet; perhaps We will do so another day.

###### [Install Rust]
```
https://www.rust-lang.org/tools/install
```

###### [Run integration tests locally]
```
$ cd /path/to/postchain-client-rust/tests/blockchain
$ ./start-postchain.bash
$ cd /path/to/postchain-client-rust
$ cargo test -- --nocapture
```

##### [TODO]
###### GTV ASN.1 encode/decode [completed] (*)
###### GTV query [completed] (*)
###### Send sign/unsign transaction [complated] (*)
###### Pooling to receive status of transaction [in-progress]
###### Add more complex tests [almost completed]

###### (*) still requires additional testing and optimization.