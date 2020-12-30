use pulldown_cmark::{Parser, Options, html, CowStr, BrokenLink, LinkType, Event};
use std::io::{self, Read, Write, Error, ErrorKind};
use std::fs;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::convert::From;

fn main() {
    let mut notebook = Notebook::new("");
    // notebook.add_ignore(Path::new("target/"));

    // notebook.add("Another test.md").unwrap();

    notebook.scan_and_add();

    println!("{:#?}", notebook.notes.keys().collect::<Vec<_>>());

    notebook.compile_note("test");
    notebook.compile_note("test2");
    // println!("{}", note.to_html_in_notebook(&notebook));
    // println!("{}", notebook.compile_note("test").unwrap());
}


struct Note {
    /// Path to the note file relative to the Notebook path
    pub path: PathBuf,
    
    // /// Last modified timestamp
    // modified: SystemTime,
    // contents: String,

    metadata: HashMap<String, String>,
}

impl Note {
    pub fn open(path: &Path) -> io::Result<Self> {
        // TODO: Maybe do reading the file lazily

        let mut file = fs::File::open(path)?;
        // let modified = file.metadata()?.modified()?;

        let mut contents = String::new();
        file.read_to_string(&mut contents)?;

        let note = Self {
            path: PathBuf::from(path),
            // modified: modified,
            metadata: {
                helpers::split_yaml_pairs(
                    helpers::strip_yaml(&mut contents)
                )
            },
            // contents,
        };

        Ok(note)
    }

    pub fn read(&self) -> String {
        // TODO: Fix this, and figure out how I'm going to add the basedir to
        // the relative path
        // Also, right now the metadata is not being read parsed at all when
        // reading
        /*
        let modified = fs::metadata(&self.rel_path).unwrap().modified().unwrap();
        if modified > self.modified {
            // Update contents of this note
            self.contents = fs::read_to_string(&self.rel_path).unwrap();

            // Update (and strip) metadata
            self.metadata = helpers::split_yaml_pairs(
                helpers::strip_yaml(&mut self.contents)
            );

            // Update modified timestamp
            self.modified = modified;
        }
        */
        fs::read_to_string(&self.path).unwrap()
    }
}

struct NotebookConfig {
    basedir: PathBuf,
    outdir: PathBuf,
    ignore: Vec<PathBuf>,
}

struct NoteCompiler {
    parse_options: Options,
}

// TODO: Add html fluff around the note, including some css
// TODO: Add flashcard support
impl NoteCompiler {
    fn to_html(&self, note: &Note, notebook: &Notebook) -> String {
        let contents = note.read();

        // Checks link reference, and creates link if the corresponding note
        // exists.
        let func = &mut |link: BrokenLink| {
            match link.link_type {
                LinkType::Shortcut => {
                    notebook.get(link.reference).and_then(|note| {
                        let path = &note.path.strip_prefix(
                            &notebook.config.basedir
                        ).unwrap().with_extension("html");
                        Some((
                            CowStr::from(String::from(path.to_str().unwrap())),
                            CowStr::from(""),
                        ))
                    })
                },
                _ => None,
            }
        };

        // Parse markdown
        let parser = Parser::new_with_broken_link_callback(
            &contents,
            self.parse_options,
            Some(func));

        let mut output = String::new();
        html::push_html(&mut output, parser);
        output
    }
}

struct Notebook {
    config: NotebookConfig,

    compiler: NoteCompiler,

    // {Note name, note}
    notes: HashMap<String, Note>,
}

impl Notebook {
    pub fn new(basedir: &str) -> Self {
        Self {
            config: NotebookConfig {
                // Set to ./ if basedir is empty
                // TODO: Remove this hack
                basedir: PathBuf::from(String::from( *[basedir, "./"].iter()
                                 .filter(|x| !x.is_empty()).take(1).next().unwrap())),

                outdir: PathBuf::from(String::from("html")),
                ignore: vec![],
            },
            compiler: NoteCompiler {
                parse_options: Options::all(),
            },
            notes: HashMap::new(),
        }
    }

    /// Add path to the ignore list
    ///
    /// `scan_dir_and_add` will ignore paths in this list
    pub fn add_ignore(&mut self, ignore: &Path) {
        if let Ok(path) = PathBuf::from(ignore).canonicalize() {
            self.config.ignore.push(path);
        }
    }

    /// Get a reference to a note in the notebook
    ///
    /// This function looks through the local list of notes. Make sure to `add`
    /// the notes before calling `find`
    pub fn get(&self, note_name: &str) -> Option<&Note> {
        self.notes.get(note_name)
    }

    /// Get mutable reference to a note in the notebook
    pub fn get_mut(&mut self, note_name: &str) -> Option<&mut Note> {
        self.notes.get_mut(note_name)
    }

    /// Add a note's absolute path to the notebook
    fn add_abs(&mut self, path: &Path) -> io::Result<&Note> {
        let note = Note::open(&path)?;
        let note_name = path.file_stem().unwrap();
        let note_name = String::from(note_name.to_str().unwrap());

        Ok(self.notes.entry(note_name).or_insert(note))
    }

    /// Add note's relative path to notebook 
    ///
    /// Searches for the filename starting in the config.basedir
    fn add(&mut self, filename: &Path) -> io::Result<&Note> {
        let mut path = PathBuf::from(&self.config.basedir);
        path.push(filename);
        self.add(&path)
    }

    /// Scans the `config.basedir` recursively and adds all markdown files found
    // TODO: See if I can return an iterator over the notes, and add helper
    // functions that can `add` and `grep` and stuff on the iterators
    pub fn scan_and_add(&mut self) {
        // TODO: Fix this clone...
        let basedir = self.config.basedir.clone();
        self.scan_dir_and_add(Path::new(&basedir));
    }

    /// Recursively scans the given `dir` for markdown files not on the
    /// ignorelist.
    fn scan_dir_and_add(&mut self, dir: &Path) {
        if self.config.ignore.contains(&dir.canonicalize().unwrap()) {
            return;
        }

        let entries = fs::read_dir(dir).unwrap()
            .map(|x| x.unwrap().path());

        let (dirs, entries): (Vec<PathBuf>, Vec<PathBuf>) =
                               entries.partition(|x| x.is_dir());

        // Recurse through directories
        for dir in dirs {
            self.scan_dir_and_add(&dir);
        }

        // Go over each markdown file in current folder
        for entry in entries.iter().filter(|x| match x.extension() {
                Some(ext) => ext == "md",
                None => false,
            })
        {
            if self.config.ignore.contains(&entry.canonicalize().unwrap()) {
                return;
            }

            self.add_abs(entry).unwrap();
        }
    }

    /// Compile a given note into HTML, and save it in the outdir at the same
    /// relative location as the markdown files.
    pub fn compile_note(&self, note_name: &str) -> io::Result<()> {
        let note = self.get(note_name)
            .ok_or(Error::new(ErrorKind::NotFound,
                    "Tried to compile an unknown note."))?;

        let outfile = self.config.basedir.join(&self.config.outdir).join(
            note.path.strip_prefix(&self.config.basedir).unwrap()
            .with_extension("html")
        );

        fs::create_dir_all(outfile.with_file_name(""))?;

        // TODO: Use buffered writer directly to file instead of first storing
        // as a String
        println!("Writing to {}", &outfile.to_str().unwrap());
        let mut file = fs::File::create(&outfile)?;
        file.write_all(self.compiler.to_html(&note, &self).as_bytes())?;
        Ok(())
    }
}

mod helpers {
    use std::collections::HashMap;

    pub fn strip_yaml(input: &mut String) -> Vec<String> {
        // Separate YAML header from markdown content
        let mut lines = input.lines().peekable();
        let output = match lines.peek() {
            Some(&"---") => {
                lines.next();

                lines.by_ref().take_while(|&line| {
                    line != "---"
                }).map(|x| x.to_string()).collect()
            },
            _ => vec![],
        };

        let new_input: String = lines.map(|x| format!("{}\n", x)).collect();

        // Update input string
        input.clear();
        input.push_str(&new_input);

        // Return yaml key value pairs per line
        output
    }

    pub fn split_yaml_pairs(input: Vec<String>) -> HashMap<String, String> {
        // TODO: Implement some YAML crate to actually parse yaml

        // Loop over each key value pair
        // TODO: See if I can clean this up some more
        let mut output: HashMap<String, String> = HashMap::new();
        for split in input.iter().map(|x| x.splitn(2, ":").map(|x| x.trim())) {
            // Unpack iterator over two values, into k, v pair
            let a: Vec<&str> = split.collect();
            match &a[..] {
                &[k, v, ..] =>
                    output.insert(String::from(k), String::from(v)),
                _ => unreachable!(),
            };
        }

        output
    }
}


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
