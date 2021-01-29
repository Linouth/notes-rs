use notes::{self, Notebook, Note};
// use notes::compiler::NoteCompiler;
use std::path::{Path, PathBuf};
use std::env;
use std::process;
use std::fs;

const DEFAULT_CONFIG: &str = "~/.config/notes.yaml";

fn main() {
    let config_file = env::args().skip(1).next()
        .unwrap_or(DEFAULT_CONFIG.to_string());
    let config_file = shellexpand::full(&config_file).unwrap();
    println!("{}", config_file);
    let config = fs::read_to_string(&*config_file).unwrap_or_else(|err| {
        println!("Error opening config file {:?}: {:?}", config_file, err);
        process::exit(1);
    });

    // TODO: This should be able to be done without converting to a Vec
    let config: Vec<String> = config.lines().map(|x| x.to_string()).collect();
    let config = notes::split_yaml_pairs(&config);

    println!("{:?}", config);

    let err_msg = |field| {
        println!("Config field missing: {}", field);
        process::exit(1);
    };
    let title = config.get("title").unwrap_or_else(|| err_msg("title"));
    let basedir = config.get("path").unwrap_or_else(|| err_msg("path"));
    let outdir = config.get("outdir").unwrap_or_else(|| err_msg("outdir"));

    let mut notebook = Notebook::new(&title, &basedir);
    notebook.add_ignore(Path::new("target/"));
    notebook.add_ignore(Path::new("assets"));
    notebook.add_ignore(Path::new("attachments"));
    notebook.add_ignore(Path::new("__layouts"));
    notebook.add_ignore(Path::new("html"));
    notebook.add_ignore(Path::new(".git"));

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
