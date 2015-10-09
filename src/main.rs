extern crate clap;
extern crate yaml_rust;

extern crate lxd;

use std::env;
use std::fs::File;
use std::io::Read;

use clap::{App, Arg, ArgMatches, SubCommand};
use yaml_rust::YamlLoader;

use lxd::{Container,LxdServer};

fn create_dividing_line(widths: &Vec<usize>) -> String {
    let mut dividing_line = String::new();
    dividing_line.push_str("+");
    for width in widths {
        dividing_line.push_str(&format!("{:-^1$}", "", width + 2));
        dividing_line.push_str("+");
    }
    dividing_line.push_str("\n");
    dividing_line
}

fn create_header_line(headers: &Vec<&str>, widths: &Vec<usize>) -> String {
    let mut header_line = String::new();
    header_line.push_str("|");
    for (n, header) in headers.iter().enumerate() {
        header_line.push_str(&format!("{:^1$}", &header, widths[n] + 2));
        header_line.push_str("|");
    }
    header_line.push_str("\n");
    header_line
}

fn create_content_line(item: &Vec<String>, widths: &Vec<usize>) -> String {
    let mut content_line = String::new();
    content_line.push_str("|");
    for (n, column_content) in item.iter().enumerate() {
        content_line.push_str(" ");
        content_line.push_str(&format!("{:1$}", &column_content, widths[n] + 1));
        content_line.push_str("|");
    }
    content_line.push_str("\n");
    content_line
}

fn format_output(headers: &Vec<&str>, items: &Vec<Vec<String>>) -> String {
    let mut widths = Vec::new();
    for header in headers {
        widths.push(header.len());
    }
    for item in items {
        for (n, column) in item.iter().enumerate() {
            if column.len() > widths[n] {
                widths[n] = column.len();
            }
        }
    }
    let dividing_line = create_dividing_line(&widths);
    let mut output_string = String::new();
    output_string.push_str(&dividing_line);
    output_string.push_str(&create_header_line(headers, &widths));
    output_string.push_str(&dividing_line);
    for item in items {
        output_string.push_str(&create_content_line(item, &widths));
    }
    output_string.push_str(&dividing_line);
    output_string
}

fn prepare_container_line(c: &Container) -> Vec<String> {
    let mut ipv4_address = String::new();
    let mut ipv6_address = String::new();
    for ip in &c.status.ips {
        if ip.protocol == "IPV4" && ip.address != "127.0.0.1" {
            ipv4_address = ip.address.clone();
        }
        if ip.protocol == "IPV6" && ip.address != "::1" {
            ipv6_address = ip.address.clone();
        }
    }
    let ephemeral = if c.ephemeral { "YES" } else { "NO" };
    vec![c.name.clone(), c.status.status.clone().to_uppercase(), ipv4_address.to_string(), ipv6_address.to_string(), ephemeral.to_string(), c.snapshot_urls.len().to_string()]
}

fn list(matches: &ArgMatches) {
    let home_dir = env::var("HOME").unwrap();
    let mut config_file = File::open(home_dir.clone() + "/.config/lxc/config.yml").unwrap();
    let mut file_contents = String::new();
    config_file.read_to_string(&mut file_contents).unwrap();
    let lxd_config = YamlLoader::load_from_str(&file_contents).unwrap();
    let default_remote = lxd_config[0]["default-remote"].as_str().unwrap();
    let remote = matches.value_of("resource").unwrap_or(default_remote);
    let lxd_server = match lxd_config[0]["remotes"][remote]["addr"].as_str() {
        Some(remote_addr) => remote_addr,
        None => panic!("No remote named {} configured", remote)
    };
    let server = LxdServer::new(
        lxd_server,
        &(home_dir.clone() + "/.config/lxc/client.crt"),
        &(home_dir.clone() + "/.config/lxc/client.key")
    );
    let headers = vec!["NAME", "STATE", "IPV4", "IPV6", "EPHEMERAL", "SNAPSHOTS"];
    let container_items = server.list_containers().iter().map(prepare_container_line).collect();
    print!("{}", format_output(&headers, &container_items));
}

fn main() {
    let matches = App::new("lxd")
        .subcommand(SubCommand::with_name("list")
                    .arg(Arg::with_name("resource")
                         .help("the resource to use")
                         .required(true)
                         .index(1)))
        .get_matches();

    match matches.subcommand_name() {
        Some("list") => list(matches.subcommand_matches("list").unwrap()),
        _ => println!("{}", matches.usage()),
    }
}
