pub trait Parser {
    fn parse(&self, content: &str) -> String;
}
