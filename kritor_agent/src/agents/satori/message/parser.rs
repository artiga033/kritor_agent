use std::collections::HashMap;

use super::{element::*, Error};
#[derive(Default)]
pub struct Parser<'a> {
    input: &'a str,
    pos: usize,
    stack: Vec<Element>,
}

impl<'a> Parser<'a> {
    pub fn new(input: &'a str) -> Self {
        Self {
            input,
            ..Default::default()
        }
    }
    pub fn parse(&mut self) -> Result<Root, Error> {
        self.stack.push(Element::empty_tag());
        while self.pos < self.input.len() {
            let c = self.cur();
            if c == '<' {
                self.parse_tag();
            } else {
                let t = self.read_until("<").unwrap_or_else(|| self.read_to_end());

                match self.current_element() {
                    Element::Plain(s) => s.push_str(t),
                    Element::Tag(e) => e
                        .children
                        .get_or_insert_with(Vec::new)
                        .push(Element::new_plain(t)),
                }
            }
        }
        if self.stack.len() == 1 {
            Ok(Root {
                root_element: self.stack.pop().unwrap(),
            })
        } else {
            Err("Unexpected end of input, tags are not closed properly.".into())
        }
    }
    fn parse_tag(&mut self) -> Option<()> {
        let _ = self.forward();
        let closing = self.cur() == '/';
        if closing {
            let _ = self.forward();
        }
        let mut tag = self.read_until(">")?;
        let _ = self.forward();
        if closing {
            // check tag matches
            if self.current_element().tag().is_some_and(|x| x.name == tag) {
                self.close_element();
                return Some(());
            } else {
                return None;
            }
        }
        let self_closing = {
            if tag.ends_with('/') {
                tag = &tag[..tag.len() - 1];
                true
            } else {
                false
            }
        };
        let mut split = tag.split_ascii_whitespace();
        let tag_name = split.next()?;
        let attributes = self.parse_attributes(split);
        let mut element = Element::with_tag_name(tag_name.to_string());
        if !attributes.is_empty() {
            let _ = element.tag_mut().unwrap().attributes.insert(attributes);
        }
        if self_closing {
            self.current_element()
                .tag_mut()
                .unwrap()
                .children_mut()
                .push(element);
        } else {
            self.stack.push(element);
        }
        Some(())
    }
    fn parse_attributes(&self, split: std::str::SplitAsciiWhitespace) -> HashMap<String, String> {
        split
            .map(|x| match x.split_once('=') {
                Some((key, value)) => (key.to_string(), {
                    let pat = &['"', '\''];
                    if value.starts_with(pat) && value.ends_with(pat) {
                        value[1..value.len() - 1].to_string()
                    } else {
                        value.to_string()
                    }
                }),
                None => (x.to_string(), String::new()),
            })
            .collect()
    }
    #[inline]
    fn current_element(&mut self) -> &mut Element {
        self.stack.last_mut().unwrap()
    }
    fn cur(&self) -> char {
        self.peek(0).unwrap()
    }
    fn peek(&self, nth: usize) -> Option<char> {
        self.input[self.pos..].chars().nth(nth)
    }
    fn forward(&mut self) -> Option<()> {
        let c = self.peek(1);
        if c.is_some() {
            self.pos += self.cur().len_utf8();
        } else {
            // got to the end
            self.pos = self.input.len();
        }
        Some(())
    }
    fn close_element(&mut self) {
        let element = self.stack.pop().unwrap();
        self.current_element()
            .tag_mut()
            .unwrap()
            .children_mut()
            .push(element);
    }

    /// Read and move the cursor until the given pattern is found.
    ///
    /// Returns the string slice between the current position and where the pattern is found.
    ///
    /// If the pattern is not found, nothing would happen. The cursor would not move either.
    fn read_until(&mut self, c: &str) -> Option<&'a str> {
        self.input[self.pos..].find(c).map(|x| {
            let start = self.pos;
            self.pos += x;
            &self.input[start..self.pos]
        })
    }
    fn read_to_end(&mut self) -> &'a str {
        let start = self.pos;
        self.pos = self.input.len();
        &self.input[start..]
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_parser() {
        let input =
            r#"1234<p><br/><b>BOLD_TEXT</b><img src="https://a33.su"/>🚀🚀++</p>Hello, World!"#;
        let mut parser = Parser::new(input);
        let root = parser.parse().unwrap();
        dbg!(&root);
        dbg!(std::mem::size_of_val(&root.root_element));
        let ser = root.root_element.serialize();
        dbg!(&ser);
        assert_eq!(ser, input);
    }
}
