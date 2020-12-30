use pulldown_cmark::{Options, BrokenLink, CowStr, LinkType, Parser, html};
use std::path::{Path, PathBuf};
use crate::{Note, Notebook};

pub enum Asset {
    Js(PathBuf),
    Css(PathBuf),
}

pub struct NoteCompiler {
    pub parse_options: Options,
    pub template: String,
    pub assets: Vec<Asset>,
}

// TODO: Add html fluff around the note, including some css
// TODO: Add flashcard support
impl NoteCompiler {
    pub fn to_html(&self, note: &Note, notebook: &Notebook) -> String {
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

    /// Compile Note to a full html buffer, with <html> tags and assets.
    pub fn to_decorated_html(&self, note: &Note, notebook: &Notebook) -> String {
        let html = self.to_html(note, notebook);

        self.template
            .replace("{assets}", &self.assets(&notebook.config.basedir))
            .replace("{content}", &html)
    }

    /// Generate string with external asset incluse lines. `basedir` is used to
    /// find the location of the file
    fn assets(&self, basedir: &Path) -> String {
        self.assets.iter().map(|asset| {

            let path = match asset {
                Asset::Js(path) | Asset::Css(path) => path,
                _ => panic!("Unknown asset type!"),
            };

            let basedir = basedir.canonicalize().unwrap();

            // Check if file exists
            if let Ok(path) = path.canonicalize() {
                // Find file in parent directories
                let mut prefix = PathBuf::new();
                for p in path.ancestors() {
                    if p == basedir {
                        break;
                    }
                    prefix.push("../");
                }

                match asset {
                    Asset::Js(path) => format!("<script src=\"{}\"></script>\n",
                        prefix.join(path).to_str().unwrap()),
                    Asset::Css(path) => format!("<link rel=\"stylesheet\" href=\"{}\">\n",
                        prefix.join(path).to_str().unwrap()),
                }
            } else {
                // File does not exist
                String::from("")
            }

        }).collect::<String>()
    }
}
