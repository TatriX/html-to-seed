//! html-to-seed: convert HTML to seed macros.
//!
//! TODO: check &nbsp;

#[macro_use]
extern crate html5ever;

use std::path::Path;
use std::string::String;

use inflector::Inflector;

use html5ever::rcdom::{Handle, NodeData, RcDom};
use html5ever::tendril::TendrilSink;
use html5ever::{parse_fragment, QualName};

use rustfmt_nightly::{
    load_config, CliOptions, Config, EmitMode,
    Input, Session,
    Verbosity,
};

fn walk(handle: &Handle) -> String {
    let node = handle;
    let mut tag = String::new();
    let mut attributes = vec![];
    let mut classes = vec![];
    let mut id = None;
    match node.data {
        NodeData::Document => {}
        NodeData::Doctype { .. } => {}
        NodeData::Text { ref contents } => {
            let text = &contents.borrow().trim().to_string();
            return if text.is_empty() {
                "".into()
            } else {
                format!("\"{}\"", text)
            };
        }
        NodeData::Comment { ref contents } => {
            let _ = contents;
            return "".into()
            // TODO
            // return format!("/* {} */", contents.escape_default());
        }
        NodeData::Element {
            ref name,
            ref attrs,
            ..
        } => {
            assert!(name.ns == ns!(html));

            tag = format!("{}", &name.local);
            if &tag == "html" {
                tag = "".into();
            }
            for attr in attrs.borrow().iter() {
                assert!(attr.name.ns == ns!());
                match &attr.name.local {
                    local_name!("class") => {
                        classes = attr
                            .value
                            .to_string()
                            .split_whitespace()
                            .map(|class| format!("\"{}\"", class))
                            .collect();
                    }
                    local_name!("id") => {
                        id = Some(attr.value.to_string());
                    }
                    name => {
                        attributes.push(format!(
                            "At::{} => \"{}\"",
                            name.to_pascal_case(),
                            attr.value.escape_default()
                        ));
                    }
                }
            }
        }

        NodeData::ProcessingInstruction { .. } => unreachable!(),
    }

    let children = node
        .children
        .borrow()
        .iter()
        .map(walk)
        .filter(|s| s != "")
        .collect::<Vec<_>>()
        .join(", ");

    if tag == "" {
        children
    } else {
        let mut components = vec![];
        if let Some(id) = id {
            components.push(format!("id!({})", id));
        }
        if !classes.is_empty() {
            components.push(format!("class![{}]", classes.join(", ")))
        }
        if !attributes.is_empty() {
            components.push(format!("attrs!{{{}}}", attributes.join(", ")))
        }
        if children != "" {
            components.push(children);
        }
        format!("{}![{}]", tag, components.join(", "))
    }
}


/// Convert string with HTML to string with seed code.
pub fn convert(input: impl Into<String>) -> String {
    let dom = parse_fragment(
        RcDom::default(),
        Default::default(),
        QualName::new(None, ns!(html), local_name!("body")),
        vec![],
    )
    .one(input.into());
    let result = walk(&dom.document);

    if !dom.errors.is_empty() {
        eprintln!("\nParse errors:");
        for err in dom.errors.iter() {
            eprintln!("    {}", err);
        }
    }
    result
}

struct NullOptions;

impl CliOptions for NullOptions {
    fn apply_to(self, _: &mut Config) {
        unreachable!();
    }
    fn config_path(&self) -> Option<&Path> {
        unreachable!();
    }
}

pub fn format(input: String) -> String {
    let (mut config, _) =
        load_config::<NullOptions>(Some(Path::new(".")), None).unwrap();

    config.set().verbose(Verbosity::Quiet);
    config.set().emit_mode(EmitMode::Stdout);

    let mut out = vec![];
    Session::new(config, Some(&mut out))
        .format(Input::Text(format!("{};", input).into()))
        .unwrap();

    String::from_utf8(out).unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple() {
        assert_eq!(convert("<div>Hi!</div>"), r#"div!["Hi!"]"#);
    }

    #[test]
    fn test_simple_nested() {
        assert_eq!(
            convert("<div>Hi!<b>Bye.</b></div>"),
            r#"div!["Hi!", b!["Bye."]]"#
        );
    }

    #[test]
    fn test_simple_attrs() {
        assert_eq!(
            convert(r#"<div class="foo">Hi!</div>"#),
            r#"div![class!["foo"], "Hi!"]"#
        );
    }

    #[test]
    fn test_ul() {
        assert_eq!(
            format(convert(include_str!("test_data/nav.html"))),
            include_str!("test_data/nav.seed"),
        );
    }
}
