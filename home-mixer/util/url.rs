pub fn percent_encode(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for &b in s.as_bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' | b'*' => {
                out.push(b as char);
            }
            _ => {
                out.push('%');
                out.push_str(&format!("{:02X}", b));
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn passes_through_unreserved() {
        assert_eq!(percent_encode("AZaz09-_.~*"), "AZaz09-_.~*");
    }

    #[test]
    fn encodes_base64_specials() {
        assert_eq!(percent_encode("a+b/c="), "a%2Bb%2Fc%3D");
    }

    #[test]
    fn encodes_space_and_other_ascii() {
        assert_eq!(percent_encode("a b&c"), "a%20b%26c");
    }
}
