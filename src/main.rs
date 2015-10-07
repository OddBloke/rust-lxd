extern crate lxd;

use lxd::list_containers;

fn format_output(headers: &Vec<&str>, items: &Vec<Vec<String>>) -> String{
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
    let mut dividing_line = "+".to_string();
    let mut header_line = "|".to_string();
    for (n, header) in headers.iter().enumerate() {
        dividing_line.push_str(&vec!["-"; widths[n] + 2].concat());
        dividing_line.push_str("+");
        header_line.push_str(&format!("{:^1$}", &header, widths[n] + 2));
        header_line.push_str("|");
    }
    let mut output_string = String::new();
    output_string.push_str(&dividing_line);
    output_string.push_str("\n");
    output_string.push_str(&header_line);
    output_string.push_str("\n");
    output_string.push_str(&dividing_line);
    output_string.push_str("\n");
    for item in items {
        output_string.push_str("|");
        for (n, column_content) in item.iter().enumerate() {
            output_string.push_str(" ");
            output_string.push_str(&format!("{:1$}", &column_content, widths[n] + 1));
            output_string.push_str("|");
        }
        output_string.push_str("\n");
    }
    output_string.push_str(&dividing_line);
    output_string
}

fn main() {
    let containers = list_containers();
    let headers = vec!["NAME", "STATE", "IPV4", "IPV6", "EPHEMERAL", "SNAPSHOTS"];
    let container_items = containers.iter().map(
        |c| {
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
        }).collect();
    println!("{}", format_output(&headers, &container_items));
}
