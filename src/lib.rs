#![feature(convert,custom_derive,plugin)]
#![plugin(serde_macros)]
extern crate hyper;
extern crate serde;
extern crate serde_json;

use std::io::Read;

use hyper::Client;
use hyper::client::IntoUrl;
use hyper::client::Response;
use hyper::net::HttpsConnector;
use hyper::net::Openssl;
use serde_json::Value;

pub struct LxdServer {
    url: String,
    cert_location: String,
    key_location: String,
}

impl LxdServer {

    pub fn new(url: &str, cert_location: &str, key_location: &str) -> LxdServer {
        LxdServer {
            url: url.to_string(),
            cert_location: cert_location.to_string(),
            key_location: key_location.to_string()
        }
    }

    fn get_client(&self) -> Client {
        let connector = HttpsConnector::new(
            Openssl::with_cert_and_key(&self.cert_location, &self.key_location).unwrap());
        Client::with_connector(connector)
    }

    pub fn get(&self, relative_url: &str) -> Response {
        let client = self.get_client();
        let url = match (self.url.clone() + relative_url).into_url() {
            Err(why) => panic!("{:?}", why),
            Ok(url) => url,
        };
        let unsent_response = client.get(url);
        match unsent_response.send() {
            Err(why) => panic!("{:?}", why),
            Ok(response) => response,
        }
    }
}

#[derive(Deserialize)]
pub struct ContainerIP {
    pub address: String,
    pub host_veth: String,
    pub interface: String,
    pub protocol: String,
}

pub struct ContainerStatus {
    pub status: String,
    pub ips: Vec<ContainerIP>,
}

pub struct Container {
    pub name: String,
    pub status: ContainerStatus,
    pub ephemeral: bool,
    pub snapshot_urls: Vec<String>,
//    pub ipv4: String,
//    pub ipv6: String,
}

impl Container {

    pub fn from_json(json: &Value) -> Container {
        let value_for_path = |path: &[&str]| {
            match json.find_path(&path) {
                Some(value) => { value }
                None => panic!("Couldn't find {} in JSON", path.join("."))
            }
        };
        let get_array_from_json = |path: &[&str]| {
            match value_for_path(&path).as_array() {
                Some(array) => { array }
                None => panic!("Couldn't find array at {}", path.join("."))
            }
        };
        let get_boolean_from_json = |path: &[&str]| {
            match value_for_path(&path).as_boolean() {
                Some(boolean) => { boolean }
                None => panic!("Couldn't find boolean at {}", path.join("."))
            }
        };
        let get_string_from_json = |path: &[&str]| {
            match value_for_path(&path).as_string() {
                Some(string) => { string.to_string() }
                None => panic!("Couldn't find string at {}", path.join("."))
            }
        };
        Container {
            ephemeral: get_boolean_from_json(&["state", "ephemeral"]),
            name: get_string_from_json(&["state", "name"]),
            status: ContainerStatus {
                status: get_string_from_json(&["state", "status", "status"]),
                ips: get_array_from_json(&["state", "status", "ips"]).iter().map(
                    |ip_value| -> ContainerIP {
                        match serde_json::from_value(ip_value.clone()) {
                            Ok(container_ip) => container_ip,
                            Err(_) => panic!("Couldn't parse a ContainerIP for {:?}", ip_value)
                        }}).collect(),
            },
            snapshot_urls: match json.find_path(&["snaps"]).unwrap().as_array() {
                Some(array) => {
                    array.iter().map(|x| {x.as_string().unwrap().to_string()}).collect()
                }
                None => {
                    vec![]
                }
            }
        }
    }
}

fn response_to_value(response: &mut Response) -> Value {
    let mut body = String::new();
    response.read_to_string(&mut body).unwrap();
    serde_json::from_str(body.as_str()).unwrap()
}

pub fn list_containers() ->  Vec<Container> {
    let server = LxdServer::new(
        "https://104.155.75.254:8443",
        "/home/daniel/.config/lxc/client.crt",
        "/home/daniel/.config/lxc/client.key"
    );
    let mut response = server.get("/1.0/containers?recursion=1");
    let payload = response_to_value(&mut response);
    let container_values = payload.find("metadata").unwrap().as_array().unwrap();
    container_values.iter().map(|v| { Container::from_json(v) }).collect()
}
