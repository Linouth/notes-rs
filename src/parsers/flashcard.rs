use std::cell::RefCell;
use regex::Regex;

use crate::parsers::Parser;

#[derive(Default)]
struct Flashcard {
    question: String,
    answer: String,
    tags: String,
}

enum FlashcardStyle {
    Oneline,
    Batch,
    Regular,
}

pub struct FlashcardParser {
    template: String,
    cards: RefCell<Vec<Flashcard>>,
}

impl FlashcardParser {
    pub fn new() -> Self {
        Self {
            cards: RefCell::new(vec![]),
            template: String::from(
r#"<div class="flashcard">
<div class="question">
{question}
</div>
<div class="answer">

{answer}
</div>
</div>"#),
        }
    }

    fn format_flashcard(&self, flashcard: &Flashcard) -> String {
        self.template.replace("{question}", &flashcard.question)
            .replace("{answer}", &flashcard.answer)
            .replace("{tags}", &flashcard.tags)
    }
}

impl Parser for FlashcardParser {
    fn parse(&self, content: &str) -> String {

        // Flashcard styles to check for
        let styles = [
            (FlashcardStyle::Oneline,
                r"(.+): (.*) #flashcard(?: ((?:#[\w\d-]+ ?)+))?"),
            (FlashcardStyle::Regular,
                r"(.+) #flashcard(?: ((?:#[\w\d-]+ ?)+))?"),
        ];

        // Flags whether a multi-line style is being parsed
        let mut active = None;
                                    
        let mut cards = self.cards.borrow_mut();

        // TODO: Find way to use string slices here
        content.lines().map(|line| {
            // Check multi-line flag
            match active {
                Some(FlashcardStyle::Regular) => {
                    let card = cards.last_mut().unwrap();

                    if line != "---" {
                        println!("Answer: {}\n", line);
                        card.answer.push_str(line);
                        card.answer.push('\n');
                    } else {
                        active = None;

                        // Return formatted flashcard
                        return self.format_flashcard(&card);
                    }

                    String::new()
                },
                None => {
                    // Check each style
                    for style in &styles {
                        let re = Regex::new(style.1).unwrap();

                        // If style maches
                        // Sets `active` if style is multi-line
                        if let Some(cap) = re.captures(line) {
                            return match style.0 {
                                FlashcardStyle::Oneline => {
                                    println!("OneLiner: {:?}", cap);

                                    // Add new card to the stack
                                    cards.push(Flashcard::default());

                                    let card = cards.last_mut().unwrap();
                                    card.question.push_str(
                                        cap.get(1).unwrap().as_str());
                                    card.answer.push_str(
                                        cap.get(2).unwrap().as_str());
                                    if let Some(tags) = cap.get(3) {
                                        card.tags.push_str(tags.as_str());
                                    }

                                    self.format_flashcard(&card)
                                },
                                FlashcardStyle::Regular => {
                                    println!("Regular question: {:?}", cap);
                                    cards.push(Flashcard::default());
                                    let card = cards.last_mut().unwrap();

                                    card.question.push_str(
                                        cap.get(1).unwrap().as_str());

                                    active = Some(FlashcardStyle::Regular);
                                    String::new()
                                },
                                FlashcardStyle::Batch => {
                                    active = Some(FlashcardStyle::Batch);
                                    String::new()
                                }
                            };
                        }
                    }

                    // Return unmodified line if it does not match any of the
                    // flashcard styles
                    String::from(line)
                },
                _ => String::from("???"),
            }
        }).fold(String::new(), |acc, line| acc + line.as_str() + "\n")
    }
}
