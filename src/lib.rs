#![feature(convert,custom_derive, plugin)]
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

pub struct Container {
    pub name: String,
    pub status: String,
//    pub ipv4: String,
//    pub ipv6: String,
//    pub ephemeral: bool,
//    pub snapshots: u8,
}

impl Container {

    pub fn from_json(json: Value) -> Container {
        let get_string_value = |path: &[&str]| {
            json.find_path(&path).unwrap().as_string().unwrap().to_string()
        };
        Container {
            name: get_string_value(&["metadata", "name"]),
            status: get_string_value(&["metadata", "status", "status"]),
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
    let mut response = server.get("/1.0/containers");
    let payload = response_to_value(&mut response);
    let container_urls = payload.find("metadata").unwrap().as_array().unwrap();
    let mut containers = Vec::new();
    for container_url in container_urls {
        let mut container_response = server.get(container_url.as_string().unwrap());
        let container_payload = response_to_value(&mut container_response);
        let container = Container::from_json(container_payload);
        containers.push(container);
    }
    containers
}
