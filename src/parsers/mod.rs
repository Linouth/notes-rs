mod flashcard;
pub use flashcard::FlashcardParser;

pub trait Parser {
    fn parse(&self, content: &str) -> String;
}
