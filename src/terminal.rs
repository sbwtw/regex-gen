
use std::sync::atomic::{AtomicUsize, Ordering};

static ID_SEQ: AtomicUsize = AtomicUsize::new(0);

#[derive(Debug)]
struct Token<'s> {
    id: usize,
    name: &'s str,
}

impl<'s> Token<'s> {
    fn new(name: &'s str) -> Token {
        Token {
            id: ID_SEQ.fetch_add(1, Ordering::SeqCst),
            name,
        }
    }
}

enum Terminal<'a> {
    Character(char),
    Token(Token<'a>),
}

#[cfg(test)]
mod test {
    use terminal::*;

    #[test]
    fn test() {
        let tok = Token::new("token");

        println!("{:?}", tok);
    }
}

