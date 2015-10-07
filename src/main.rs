extern crate lxd;

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

fn main() {
    let server = LxdServer::new(
        "https://104.155.75.254:8443",
        "/home/daniel/.config/lxc/client.crt",
        "/home/daniel/.config/lxc/client.key"
    );
    let headers = vec!["NAME", "STATE", "IPV4", "IPV6", "EPHEMERAL", "SNAPSHOTS"];
    let container_items = server.list_containers().iter().map(prepare_container_line).collect();
    print!("{}", format_output(&headers, &container_items));
}
