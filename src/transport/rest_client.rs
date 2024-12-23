extern crate serde_json;
extern crate url;

use reqwest::{header::CONTENT_TYPE, Client};
use url::Url;

use serde_json::Value;
use std::{error::Error, time::Duration};

use crate::utils::transaction::Transaction;

pub struct RestClient<'a> {
    pub node_url: Vec<&'a str>,
    pub time_out: u64,
}

#[derive(Debug)]
pub enum RestResponse {
    String(String),
    Json(Value),
    Bytes(Vec<u8>),
}

#[derive(PartialEq, Eq)]
pub enum RestRequestMethod {
    GET,
    POST,
}

impl<'a> Default for RestClient<'a> {
    fn default() -> Self {
        return RestClient {
            node_url: vec!["http://localhost:7740"],
            time_out: 30,
        };
    }
}

#[derive(Debug)]
enum TypeError {
    FromReqClient,
    FromRestApi,
}

#[derive(Debug)]
pub struct RestError {
    status_code: Option<String>,
    error_str: Option<String>,
    error_json: Option<Value>,
    type_error: TypeError,
}

impl Error for RestError {}

impl Default for RestError {
    fn default() -> Self {
        return RestError {
            status_code: None,
            error_str: None,
            error_json: None,
            type_error: TypeError::FromRestApi,
        };
    }
}

impl std::fmt::Display for RestError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut hsc = "N/A".to_string();
        let mut err_str = "N/A".to_string();

        if let Some(val) = &self.status_code {
            hsc = val.clone();
        }

        if let Some(val) = &self.error_str {
            err_str = val.clone();
        }

        write!(f, "{:?} {} {}", self.type_error, hsc, err_str)
    }
}

impl<'a> RestClient<'a> {
    pub async fn get_nodes_from_directory(&self, brid: &str) -> Result<Vec<String>, RestError> {
        let directory_brid = self.get_blockchain_rid(0).await?;

        let path_segments = &["query", &directory_brid];
        let mut query_params = vec![
            ("type", "cm_get_blockchain_api_urls"),
            ("blockchain_rid", brid),
        ];
        let query_body_json = None;
        let query_body_raw = None;

        let resp = self
            .postchain_rest_api(
                RestRequestMethod::GET,
                Some(path_segments),
                Some(&mut query_params),
                query_body_json,
                query_body_raw,
            )
            .await;

        match resp {
            Ok(val) => match val {
                RestResponse::Json(json_val) => {
                    let list_of_nodes = json_val
                        .as_array()
                        .unwrap()
                        .iter()
                        .filter_map(|value| value.as_str().map(String::from))
                        .collect();
                    Ok(list_of_nodes)
                }
                RestResponse::String(str_val) => Ok(vec![str_val]),
                _ => Ok(vec!["nop".to_string()]),
            },
            Err(error) => Err(error),
        }
    }

    pub async fn get_blockchain_rid(&self, blockchain_iid: u8) -> Result<String, RestError> {
        let resp: Result<RestResponse, RestError> = self
            .postchain_rest_api(
                RestRequestMethod::GET,
                Some(&[&format!("/brid/iid_{}", blockchain_iid)]),
                None,
                None,
                None,
            )
            .await;

        if let Err(error) = resp {
            return Err(error);
        }

        let resp_val: RestResponse = resp.unwrap();

        match resp_val {
            RestResponse::String(val) => Ok(val.to_string()),
            _ => Ok("".to_string()),
        }
    }

    pub fn print_error(&self, error: &RestError, ignore_all_errors: bool) -> bool {
        println!(">> Error(s)");

        if let Some(error_str) = &error.error_str {
            println!("{}", error_str);
        } else {
            let val = &error.error_json.as_ref().unwrap();
            let pprint = serde_json::to_string_pretty(val).unwrap();
            println!("{}", pprint);
        }

        if ignore_all_errors {
            println!("Allow ignore this error");
            return false
        }

        true
    }

    pub fn update_node_urls(&mut self, node_urls: &'a Vec<String>) {
        self.node_url = node_urls.iter().map(String::as_str).collect();
    }

    // Submit transaction
    // POST /tx/{blockchainRid}
    pub async fn send_transaction(&self, tx: &Transaction<'a>) -> Result<RestResponse, RestError> {
        let txe = tx.gvt_hex_encoded();

        let resq_body: serde_json::Map<String, Value> =
            vec![("tx".to_string(), serde_json::json!(txe))]
                .into_iter()
                .collect();

        let blockchain_rid = hex::encode(tx.blockchain_rid.clone()).as_str().to_owned(); 

        self
            .postchain_rest_api(
                RestRequestMethod::POST,
                Some(&["tx", &blockchain_rid]),
                None,
                Some(serde_json::json!(resq_body)),
                None,
            )
            .await
    }

    // Make a query with GTV encoded response
    // POST /query_gtv/{blockchainRid}
    pub async fn query(
        &self,
        brid: &str,
        query_prefix: Option<&str>,
        query_type: &'a str,
        query_params: Option<&'a mut Vec<(&'a str, &'a str)>>,
        query_args: Option<&'a mut Vec<(&str, crate::utils::params::Params<'_>)>>,
    ) -> Result<RestResponse, RestError> {
        let mut query_prefix_str = "query_gtv";

        if let Some(val) = query_prefix {
            query_prefix_str = val;
        }

        let encode_str = crate::encoding::gtv::encode(query_type, query_args);

        self.postchain_rest_api(
            RestRequestMethod::POST,
            Some(&[query_prefix_str, brid]),
            query_params,
            None,
            Some(encode_str),
        )
        .await
    }

    async fn postchain_rest_api(
        &self,
        method: RestRequestMethod,
        path_segments: Option<&[&str]>,
        query_params: Option<&'a mut Vec<(&'a str, &'a str)>>,
        query_body_json: Option<Value>,
        query_body_raw: Option<Vec<u8>>,
    ) -> Result<RestResponse, RestError> {
        // TODO: If request on a node is failed then try another node.
        let node_index: usize = 0;
        let mut url = Url::parse(&self.node_url[node_index]).unwrap();

        if let Some(ps) = path_segments {
            if !ps.is_empty() {
                let psj = ps.join("/");
                url.set_path(&psj);
            }
        }

        if let Some(qp) = query_params {
            if !qp.is_empty() {
                for (name, value) in qp {
                    url.query_pairs_mut().append_pair(name, value);
                }
            }
        }

        if method == RestRequestMethod::POST
            && query_body_json.is_none()
            && query_body_raw.is_none()
        {
            return Err(RestError {
                type_error: TypeError::FromRestApi,
                error_str: Some("Error: POST request need a body [json or binary].".to_string()),
                status_code: None,
                ..Default::default()
            });
        }

        let rest_client = Client::new();

        let req_result = match method {
            RestRequestMethod::GET => {
                rest_client
                    .get(url.clone())
                    .timeout(Duration::from_secs(self.time_out))
                    .send()
                    .await
            }

            RestRequestMethod::POST => {
                if let Some(qb) = query_body_json {
                    rest_client
                        .post(url.clone())
                        .timeout(Duration::from_secs(self.time_out))
                        .json(&qb)
                        .send()
                        .await
                } else {
                    let r_body = reqwest::Body::from(query_body_raw.unwrap());
                    rest_client
                        .post(url.clone())
                        .timeout(Duration::from_secs(self.time_out))
                        .body(r_body)
                        .send()
                        .await
                }
            }
        };

        match req_result {
            Ok(resp) => {
                let http_status_code = resp.status().to_string();
                let http_resp_header = resp.headers().get(CONTENT_TYPE).unwrap().to_str().unwrap();
                let json_resp = http_resp_header.contains("application/json");
                let octet_stream_resp = http_resp_header.contains("application/octet-stream");

                if http_status_code.starts_with('4') || http_status_code.starts_with('5') {
                    let mut err = RestError {
                        status_code: Some(http_status_code),
                        type_error: TypeError::FromRestApi,
                        ..Default::default()
                    };

                    if json_resp {
                        let error_json = resp.json().await.unwrap();
                        err.error_json = Some(error_json);
                    } else {
                        let error_str = resp.text().await.unwrap();
                        err.error_str = Some(error_str);
                    }

                    return Err(err);
                }

                let rest_resp: RestResponse;

                if json_resp {
                    let val = resp.json().await.unwrap();
                    rest_resp = RestResponse::Json(val);
                } else if octet_stream_resp {
                    let bytes = resp.bytes().await.unwrap();
                    rest_resp = RestResponse::Bytes(bytes.to_vec());
                } else {
                    let val = resp.text().await.unwrap();
                    rest_resp = RestResponse::String(val);
                }

                Ok(rest_resp)
            }
            Err(error) => Err(RestError {
                error_str: Some(error.to_string()),
                type_error: TypeError::FromReqClient,
                ..Default::default()
            }),
        }
    }
}
