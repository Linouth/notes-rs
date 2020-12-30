use notes::Notebook;
use std::path::Path;

fn main() {
    let mut notebook = Notebook::new("");
    notebook.add_ignore(Path::new("target/"));

    // notebook.add("Another test.md").unwrap();

    notebook.scan_and_add();

    notebook.compile_all().unwrap();
}

/*
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file() {
        let filename = "Another test.md";

        let mut note = match Note::open(filename) {
            Err(e) => panic!("{:?}", e),
            Ok(n) => n,
        };

        assert_eq!(note.read(), "\n# Another test\n\nThis is another test file\n");
        assert_eq!(note.metadata["tags"], "#test");

        let html = r#"<h1>Another test</h1>
<p>This is another test file</p>
"#;
        assert_eq!(note.to_html(), html);
    }

    #[test]
    #[should_panic]
    fn test_invalid_file() {
        let filename = "blah.md";

        let _note = Note::open(filename).unwrap();
    }

    #[test]
    fn strip_yaml() {
        let mut input = String::from(r#"---
key: value1
key2: value2
---
Test
"#);
        let output = helpers::strip_yaml(&mut input);
        
        assert_eq!(output, vec!["key: value1", "key2: value2"]);
        assert_eq!(&input, "Test\n");
    }
}
*/
