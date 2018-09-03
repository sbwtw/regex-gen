
use std::convert::From;
use std::str::Chars;
use std::iter::Peekable;
use std::string::ToString;

#[derive(Debug, PartialEq)]
enum RegexUnit {
    Character(char),
    CharacterRange(char, char),
    Characters(Vec<RegexUnit>),
    Items(Vec<RegexItem>),
}

#[derive(Debug, PartialEq)]
enum RegexAnnotation {
    StandAlone,
    OneOrZero,      // '?'
    GreaterZero,    // '+'
    AnyOccurs,      // '*'
}

#[derive(Debug, PartialEq)]
pub struct RegexItem {
    unit: RegexUnit,
    annotation: RegexAnnotation,
}

impl<'s> From<&'s str> for RegexItem {
    fn from(s: &'s str) -> RegexItem {
        RegexParser {
            input: s.chars().peekable(),
        }.parse().unwrap()
    }
}

impl ToString for RegexUnit {
    fn to_string(&self) -> String {
        let mut r = String::new();

        match self {
            RegexUnit::Character(c) => r.push(*c),
            RegexUnit::CharacterRange(s, e) => {
                r.push(*s);
                r.push('-');
                r.push(*e);
            },
            RegexUnit::Characters(list) => {
                r.push('[');
                for i in list {
                    r.push_str(&i.to_string());
                }
                r.push(']');
            },
            RegexUnit::Items(list) => {
                r.push('(');
                for i in list {
                    r.push_str(&i.to_string());
                }
                r.push(')');
            },
        }

        r
    }
}

impl ToString for RegexItem {
    fn to_string(&self) -> String {
        let mut r = String::new();

        r.push_str(&self.unit.to_string());

        match self.annotation {
            RegexAnnotation::AnyOccurs => r.push('*'),
            RegexAnnotation::OneOrZero => r.push('?'),
            RegexAnnotation::GreaterZero => r.push('+'),
            _ => {},
        }

        r
    }
}

type RegexParserError = ();
type RegexParserResult = Result<RegexItem, RegexParserError>;

struct RegexParser<'s> {
    input: Peekable<Chars<'s>>,
}

impl<'s> RegexParser<'s> {
    fn parse(&mut self) -> RegexParserResult {
        let mut items = vec![];

        while let Ok(item) = self.dispatch() {
            items.push(item);
        }

        //assert_eq!(self.parse_annotation(), RegexAnnotation::StandAlone);

        Ok(RegexItem {
            unit: RegexUnit::Items(items),
            annotation: RegexAnnotation::StandAlone,
        })
    }

    fn dispatch(&mut self) -> RegexParserResult {
        if let Some(c) = self.input.peek().map(|x| x.clone())
        {
            match c {
                '[' => self.parse_character_group(),
                '(' => self.parse_item_group(),
                _ => self.parse_character(),
            }
        } else {
            Err(())
        }
    }

    fn parse_character(&mut self) -> RegexParserResult {
        let c = self.input.next().unwrap();

        Ok(RegexItem {
            unit: RegexUnit::Character(c),
            annotation: self.parse_annotation(),
        })
    }

    fn parse_character_group(&mut self) -> RegexParserResult {
        assert_eq!(Some('['), self.input.next());
        let mut items = vec![];

        // special process for first '-'
        if let Some('-') = self.input.peek() {
            self.input.next();

            items.push(RegexUnit::Character('-'));
        }

        loop {
            match self.input.next().map(|x| x.clone()) {
                Some('\\') => {
                    match self.input.peek() {
                        Some('d') => {
                            self.input.next();

                            items.push(RegexUnit::CharacterRange('0', '9'));
                        }
                        Some('\\') | Some('[') | Some(']') => {
                            items.push(RegexUnit::Character(self.input.next().unwrap()));
                        },
                        _ => return Err(()),
                    }                    
                },
                Some('a') => {
                    if let Some('-') = self.input.peek() {
                        self.input.next();
                        match self.input.next() {
                            Some('z') => items.push(RegexUnit::CharacterRange('a', 'z')),
                            _ => return Err(()),
                        }
                    } else {
                        items.push(RegexUnit::Character('a'));
                    }
                },
                Some('A') => {
                    if let Some('-') = self.input.peek() {
                        self.input.next();
                        match self.input.next() {
                            Some('Z') => items.push(RegexUnit::CharacterRange('A', 'Z')),
                            _ => return Err(()),
                        }
                    } else {
                        items.push(RegexUnit::Character('A'));
                    }
                },
                Some('0') => {
                    if let Some('-') = self.input.peek() {
                        self.input.next();
                        match self.input.next() {
                            Some('9') => items.push(RegexUnit::CharacterRange('0', '9')),
                            _ => return Err(()),
                        }
                    } else {
                        items.push(RegexUnit::Character('0'));
                    }
                },
                Some(']') => {
                    return Ok(RegexItem {
                        unit: RegexUnit::Characters(items),
                        annotation: self.parse_annotation(),
                    })
                },
                Some(c) => {
                    items.push(RegexUnit::Character(c));
                },
                None => return Err(()),
            }
        }
    }

    fn parse_item_group(&mut self) -> RegexParserResult {
        self.input.next();
        let mut items = vec![];

        loop {
            match self.input.peek().map(|x| x.clone()) {
                Some(')') => {
                    self.input.next();

                    return Ok(RegexItem {
                        unit: RegexUnit::Items(items),
                        annotation: self.parse_annotation(),
                    })
                },
                Some(_) => items.push(self.dispatch()?),
                None => return Err(()),
            }
        }
    }

    fn parse_annotation(&mut self) -> RegexAnnotation {
        let r = match self.input.peek() {
            Some('?') => RegexAnnotation::OneOrZero,
            Some('+') => RegexAnnotation::GreaterZero,
            Some('*') => RegexAnnotation::AnyOccurs,
            _ => return RegexAnnotation::StandAlone,
        };

        self.input.next();
        r
    }

}

#[cfg(test)]
mod test {

    use regex_gen::*;

    #[test]
    fn test() {
        let r1: RegexItem = r#"a[-a\\bd\[\]\d]+"#.into();
        let r2: RegexItem = r#"a[-a\\bd\[\]0-9]+"#.into();

        assert_eq!(r1, r2);
    }
}

