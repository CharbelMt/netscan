pub fn parse_ports(input: &str) -> Result<Vec<u16>, String> {
    let ports = input.split(','); // split input on commas

    let mut parsed_ports = vec![]; // vector to store valid ports

    // parse each port
    for port in ports {
        // parse port ranges (separated by a "-")
        match port.split_once('-') {
            // if valid port range
            Some((start, end)) => {
                // extract start port
                let s = match parse_port(start) {
                    Ok(x) => x,
                    Err(e) => {
                        return Err(e);
                    }
                };

                // extract end port
                let e = match parse_port(end) {
                    Ok(x) => x,
                    Err(e) => {
                        return Err(e);
                    }
                };

                // ensure reversed ranges are not accepted
                if s > e {
                    return Err(String::from(
                        "Start port cannot be greater than end port in a range.",
                    ));
                }

                // push the ports into the parsed ports vector
                for i in s..=e {
                    parsed_ports.push(i);
                }
            }

            // if not a range, parse normally
            None => {
                parsed_ports.push(match parse_port(port) {
                    Ok(x) => x,
                    Err(e) => {
                        return Err(e);
                    }
                });
            }
        };
    }

    parsed_ports.sort_unstable(); // sort the vector
    parsed_ports.dedup(); // remove duplicates

    Ok(parsed_ports)
}

fn parse_port(s: &str) -> Result<u16, String> {
    // try to parse a string to u16
    let number = match s.trim().parse::<u16>() {
        // if valid
        Ok(x) => {
            // ensure not 0
            if x == 0 {
                return Err(String::from("Port cannot be zero"));
            }

            x
        }

        // if invalid, exit the function
        Err(e) => {
            return Err(e.to_string());
        }
    };

    Ok(number)
}

#[cfg(test)]
mod tests {
    use super::*;

    // succeed

    #[test]
    fn single_port() {
        assert_eq!(parse_ports("80"), Ok(vec![80]));
    }

    #[test]
    fn range_is_inclusive() {
        assert_eq!(parse_ports("20-23"), Ok(vec![20, 21, 22, 23]));
    }

    #[test]
    fn output_is_sorted_and_deduplicated() {
        assert_eq!(parse_ports("443,22"), Ok(vec![22, 443])); //sorted
        assert_eq!(parse_ports("80,80,79-81"), Ok(vec![79, 80, 81])); // de-duplicated
    }

    #[test]
    fn multiple_single_ports() {
        assert_eq!(parse_ports("22,80,443"), Ok(vec![22, 80, 443]));
    }

    #[test]
    fn single_ports_with_range() {
        assert_eq!(
            parse_ports("22,80,100-102"),
            Ok(vec![22, 80, 100, 101, 102])
        );
    }

    #[test]
    fn trim_edges() {
        assert_eq!(parse_ports(" 22 , 80 "), Ok(vec![22, 80]));
    }

    #[test]
    fn max_range() {
        let ports = parse_ports("1-65535").unwrap();
        assert_eq!(ports.len(), 65535);
        assert_eq!(ports.first(), Some(&1));
        assert_eq!(ports.last(), Some(&65535));
    }

    // fail

    #[test]
    fn rejects_invalid_input() {
        let bad = [
            "",
            "abc",
            "22,,80",
            "0",
            "65536", // greater than u16 maximum. fails via parse automatically
            "100-50",
            "80-",
            "-80",
            "1-2-3",
            "0-100",
            "65535-65536",
        ];
        for input in bad {
            assert!(
                parse_ports(input).is_err(),
                "should have rejected {input:?}"
            );
        }
    }
}
