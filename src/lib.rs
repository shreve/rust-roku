extern crate curl;

use std::net::UdpSocket;
use std::time::Duration;
use curl::easy::Easy;

static ADDR: &str = "239.255.255.250:1900";
static SEARCH: &str = r#"M-SEARCH * HTTP/1.1
Host: 239.255.255.250:1900
Man: "ssdp:discover"
ST: roku:ecp
"#;

fn parse_response(resp: &str) -> Option<&str> {
    // Search the whole thing for Roku. If there, we continue.
    if ! resp.contains("Roku") {
        return None;
    }

    // Find the line that provides LOCATION.
    let line = resp.lines()
        .filter(|line| line.contains("LOCATION") )
        .last();

    // Pull out the location value from that line.
    match line {
        None => None,
        Some(string) => {
            match string.find(": ") {
                None => None,
                Some(index) => {
                    Some(&string[(index + 2)..])
                }
            }
        }
    }
}

#[derive(std::fmt::Debug)]
pub struct Client {
    addr: String
}

impl Client {
    pub fn new(address: &str) -> Self {
        Self { addr: address.to_string() }
    }

    pub fn discover() -> Self {
        let socket = UdpSocket::bind("0.0.0.0:0")
            .expect("Couldn't bind to address");

        socket.send_to(SEARCH.as_bytes(), ADDR)
            .expect("Couldn't send data");

        socket.set_read_timeout(Some(Duration::new(5, 0)))
            .expect("Couldn't set timeout");

        loop {
            // Max size of a udp packet
            let mut buf = [0; 8192];
            let (amt, _src) = socket.recv_from(&mut buf)
                .expect("Couldn't read data");

            // Grab the written bytes into a str
            let response = std::str::from_utf8(&buf[..amt])
                .expect("Couldn't read bytes into string");

            // If we can find a URL in the response, use it.
            match parse_response(response) {
                None => continue,
                Some(url) => return Client::new(url)
            }
        }
    }

    pub fn keypress(&self, key: &str) -> Result<bool, &str> {
        let mut path = "keypress/".to_owned();
        path.push_str(key);
        match self.post(path.as_str()) {
            Ok(_) => Ok(true),
            Err(msg) => Err(msg)
        }
    }

    pub fn device_info(&self) -> Result<String, &str> {
        self.get("query/device-info")
    }

    fn get(&self, path: &str) -> Result<String, &str> {
        let mut out = Vec::new();

        let mut url = self.addr.as_str().to_owned();
        url.push_str(path);

        {
            // Reduce the scope of the request so the mutable borrow
            // of out can be dropped before converting to string at the end.
            let mut req = Easy::new();
            req.url(&url).unwrap();

            let mut transfer = req.transfer();
            transfer.write_function(|data| {
                out.extend_from_slice(data);
                Ok(data.len())
            }).unwrap();
            transfer.perform().unwrap();
        }

        Ok(String::from_utf8(out).unwrap())
    }

    fn post(&self, path: &str) -> Result<String, &str> {
        let mut url = self.addr.as_str().to_owned();
        url.push_str(path);

        let mut req = Easy::new();
        req.url(&url).unwrap();
        req.post(true).unwrap();
        req.post_field_size(0).unwrap();
        req.transfer().perform().unwrap();

        if req.response_code().unwrap() == 200 {
            Ok(String::new())
        } else {
            Err("Unable to complete POST request")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let client = Client::discover();
        // match client.keypress("Select") {
        //     Ok(string) => println!("{:?}", string),
        //     Err(msg) => println!("{}", msg)
        // }
    }
}
