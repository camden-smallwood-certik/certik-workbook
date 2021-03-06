#[macro_use]
extern crate serde_derive;

pub mod command;
pub mod html;
pub mod report;

use crate::{
    html::HtmlElement,
    report::Finding
};

use std::collections::HashMap;

#[derive(Debug)]
pub struct StateData {
    pub initialized: bool,
    pub current_finding_id: usize,
    pub checklist: Vec<(bool, String)>,
    pub findings: HashMap<usize, Finding>,
    pub copied_finding: Option<Finding>
}

impl StateData {
    pub fn new() -> Self {
        Self {
            initialized: false,
            current_finding_id: 0,
            checklist: vec![],
            findings: HashMap::new(),
            copied_finding: None
        }
    }
}

fn main() {
    // Initialize the local state data
    let mut state = StateData::new();

    // Build the web view
    let view = web_view::builder()
        .title("CertiK Workbook")
        .content(web_view::Content::Html(include_str!("../html/index.html")))
        .size(1024, 640)
        .resizable(true)
        .debug(true)
        .user_data(())
        .invoke_handler(|view, arg| {
            println!("Command: {}", arg);
            
            // Tokenize the input command and parameters
            let tokens = tokenize_command_string(arg);
            let mut iter = tokens.iter();

            // Handle the input command
            match iter.next().unwrap().as_str() {
                "load_active_workbook" => load_active_workbook(view, &mut state)?,
                "load_workbook" => load_workbook(view, &mut state)?,
                "save_workbook" => save_workbook(&mut state)?,
                "import_markdown" => import_markdown(view, &mut state)?,
                "export_markdown" => export_markdown(&mut state)?,
                "export_pdf" => export_pdf(view)?,
                "create_checklist_entry" => create_checklist_entry(view, &mut state)?,
                "remove_checklist_entry" => remove_checklist_entry(view, &mut state, iter.next().unwrap().parse().unwrap())?,
                "clear_checklist_entries" => clear_checklist_entries(view, &mut state)?,
                "set_checklist_entry_checked" => set_checklist_entry_checked(view, &mut state, iter.next().unwrap().parse().unwrap(), iter.next().unwrap().parse().unwrap())?,
                "set_checklist_entry_text" => set_checklist_entry_text(view, &mut state, iter.next().unwrap().parse().unwrap(), iter.next().unwrap())?,
                "create_finding" => create_finding(view, &mut state)?,
                "copy_finding" => copy_finding(view, &mut state, iter.next().unwrap().parse().unwrap())?,
                "paste_finding" => paste_finding(view, &mut state)?,
                "remove_finding" => remove_finding(view, &mut state, iter.next().unwrap().parse().unwrap())?,
                "clear_findings" => clear_findings(view, &mut state)?,
                "set_finding_title" => set_finding_title(view, &mut state, iter.next().unwrap().parse().unwrap(), iter.next().unwrap())?,
                "set_finding_type" => set_finding_type(view, &mut state, iter.next().unwrap().parse().unwrap(), iter.next().unwrap())?,
                "set_finding_severity" => set_finding_severity(view, &mut state, iter.next().unwrap().parse().unwrap(), iter.next().unwrap())?,
                "set_finding_location" => set_finding_location(view, &mut state, iter.next().unwrap().parse().unwrap(), iter.next().unwrap())?,
                "set_finding_description" => set_finding_description(view, &mut state, iter.next().unwrap().parse().unwrap(), iter.next().unwrap())?,
                "set_finding_recommendation" => set_finding_recommendation(view, &mut state, iter.next().unwrap().parse().unwrap(), iter.next().unwrap())?,
                "set_finding_alleviation" => set_finding_alleviation(view, &mut state, iter.next().unwrap().parse().unwrap(), iter.next().unwrap())?,
                command => view.eval(format!("alert(\"Command not implemented: '{}'\")", command).as_str())?
            }

            // Verify all parameters were used
            assert!(iter.next().is_none());

            Ok(())
        })
        .build()
        .unwrap();

    // Set up the main menu for macOS
    #[cfg(target_os = "macos")] unsafe {
        use cocoa::base::{selector, nil};
        use cocoa::foundation::{NSAutoreleasePool, NSString};
        use cocoa::appkit::{
            NSApp, NSApplication, NSMenu, NSMenuItem, NSRunningApplication, NSEventModifierFlags,
            NSApplicationActivateIgnoringOtherApps, NSApplicationActivationPolicyRegular
        };
        
        let _ = NSAutoreleasePool::new(nil);
 
        //----------------------------------
 
        let app = NSApp();
        app.setActivationPolicy_(NSApplicationActivationPolicyRegular);
 
        //----------------------------------
 
        let menu_bar = NSMenu::new(nil).autorelease();

        //----------------------------------
 
        let app_menu_item = NSMenuItem::new(nil).autorelease();
        menu_bar.addItem_(app_menu_item);

        //----------------------------------
 
        app.setMainMenu_(menu_bar);
 
        //----------------------------------
 
        let app_menu = NSMenu::new(nil).autorelease();
        
        let undo_title = NSString::alloc(nil).init_str("Undo");
        let undo_action = selector("undo:");
        let undo_key = NSString::alloc(nil).init_str("z");
        let undo_item = NSMenuItem::alloc(nil)
            .initWithTitle_action_keyEquivalent_(undo_title, undo_action, undo_key)
            .autorelease();
        app_menu.addItem_(undo_item);

        let redo_title = NSString::alloc(nil).init_str("Redo");
        let redo_action = selector("redo:");
        let redo_key = NSString::alloc(nil).init_str("z");
        let redo_item = NSMenuItem::alloc(nil)
            .initWithTitle_action_keyEquivalent_(redo_title, redo_action, redo_key)
            .autorelease();
        redo_item.setKeyEquivalentModifierMask_(NSEventModifierFlags::NSCommandKeyMask | NSEventModifierFlags::NSShiftKeyMask);
        app_menu.addItem_(redo_item);

        let select_all_title = NSString::alloc(nil).init_str("Select All");
        let select_all_action = selector("selectAll:");
        let select_all_key = NSString::alloc(nil).init_str("a");
        let select_all_item = NSMenuItem::alloc(nil)
            .initWithTitle_action_keyEquivalent_(select_all_title, select_all_action, select_all_key)
            .autorelease();
        app_menu.addItem_(select_all_item);

        let cut_title = NSString::alloc(nil).init_str("Cut");
        let cut_action = selector("cut:");
        let cut_key = NSString::alloc(nil).init_str("x");
        let cut_item = NSMenuItem::alloc(nil)
            .initWithTitle_action_keyEquivalent_(cut_title, cut_action, cut_key)
            .autorelease();
        app_menu.addItem_(cut_item);

        let copy_title = NSString::alloc(nil).init_str("Copy");
        let copy_action = selector("copy:");
        let copy_key = NSString::alloc(nil).init_str("c");
        let copy_item = NSMenuItem::alloc(nil)
            .initWithTitle_action_keyEquivalent_(copy_title, copy_action, copy_key)
            .autorelease();
        app_menu.addItem_(copy_item);

        let paste_title = NSString::alloc(nil).init_str("Paste");
        let paste_action = selector("paste:");
        let paste_key = NSString::alloc(nil).init_str("v");
        let paste_item = NSMenuItem::alloc(nil)
            .initWithTitle_action_keyEquivalent_(paste_title, paste_action, paste_key)
            .autorelease();
        app_menu.addItem_(paste_item);

        let quit_title = NSString::alloc(nil).init_str("Quit");
        let quit_action = selector("terminate:");
        let quit_key = NSString::alloc(nil).init_str("q");
        let quit_item = NSMenuItem::alloc(nil)
            .initWithTitle_action_keyEquivalent_(quit_title, quit_action, quit_key)
            .autorelease();
        app_menu.addItem_(quit_item);

        app_menu_item.setSubmenu_(app_menu);

        //----------------------------------
 
        let current_app = NSRunningApplication::currentApplication(nil);
        current_app.activateWithOptions_(NSApplicationActivateIgnoringOtherApps);

        //----------------------------------
 
        app.run();
    }

    view.run().unwrap()
}

fn tokenize_command_string(string: &str) -> Vec<String> {
    let mut is_string = false;
    let mut tokens = vec![];
    let mut token = String::new();

    // Create an iterator to loop over each character
    let mut iter = string.chars();

    // Build each token using the character iterator
    while let Some(c) = iter.next() {
        if c.is_whitespace() {
            if is_string {
                // Push all whitespace characters inside a string to the token
                token.push(c);
            } else if token.len() != 0 {
                // Push the current token to the vector if token is not empty
                tokens.push(token.replace("''", "\""));
                token = String::new();
            }
        } else if c == '"' {
            // Flip string mode
            is_string = !is_string;
        } else {
            // Push the character to the token
            token.push(c);
        }
    }

    // Push the last token (if any)
    if token.len() != 0 {
        tokens.push(token.replace("''", "\""));
    }

    tokens
}

fn load_active_workbook<'a>(view: &mut web_view::WebView<'a, ()>, state: &mut StateData) -> web_view::WVResult {
    let mut findings = vec![];
    
    for finding in state.findings.values() {
        findings.push(finding.clone());
    }

    findings.sort_by(|a, b| a.id.cmp(&b.id));

    for ref finding in findings {
        add_finding_to_web_view(view, finding)?;
    }

    Ok(())
}

fn load_workbook<'a>(view: &mut web_view::WebView<'a, ()>, state: &mut StateData) -> web_view::WVResult {
    if let Some(path) = tinyfiledialogs::open_file_dialog("Select a JSON workbook", "workbook.json", None) {
        use std::fs;

        match fs::read_to_string(path) {
            Err(error) => {
                return Err(web_view::Error::Custom(Box::new(error)))
            }

            Ok(json) => {
                match serde_json::from_str(json.as_str()) {
                    Err(error) => {
                        return Err(web_view::Error::Custom(Box::new(error)))
                    }

                    Ok(report::Report { checklist, mut findings, .. }) => {
                        clear_checklist_entries(view, state)?;
                        clear_findings(view, state)?;

                        for (index, entry) in checklist.iter().enumerate() {
                            state.checklist.push(entry.clone());
                            add_checklist_entry_to_web_view(view, index, (entry.0, entry.1.as_str()))?;
                        }
                        
                        findings.sort_by(|lhs, rhs| lhs.id.cmp(&rhs.id));

                        state.current_finding_id = findings.len();
                        
                        for (_, finding) in findings.iter().enumerate() {
                            assert!(state.findings.insert(finding.id, finding.clone()).is_none());                        
                            add_finding_to_web_view(view, &finding)?;
                        }
                    }
                }
            }
        }
    }

    Ok(())
}

fn save_workbook<'a>(state: &mut StateData) -> web_view::WVResult {
    if let Some(path) = tinyfiledialogs::save_file_dialog("Select a JSON workbook", "workbook.json") {
        use std::{fs::File, io::Write};

        let mut file = match File::create(path) {
            Err(error) => return Err(web_view::Error::Custom(Box::new(error))),
            Ok(file) => file
        };

        let mut report = report::Report {
            title: "Report Title".to_string(),
            auditors: vec![
                report::Auditor {
                    name: "Camden Smallwood".to_string(),
                    email: "camden.smallwood@certik.org".to_string()
                }
            ],
            start_time: "Oct. 12, 2020".to_string(),
            delivery_time: "Oct. 19, 2020".to_string(),
            repository: "Repository URL".to_string(),
            commit_hashes: vec!["Commit Hash 1".to_string(), "Commit Hash 2".to_string()],
            checklist: vec![],
            overview: "Executive Overview".to_string(),
            findings: vec![]
        };

        for entry in &state.checklist {
            report.checklist.push(entry.clone());
        }

        for (_, finding) in &state.findings {
            report.findings.push(finding.clone());
        }

        report.findings.sort_by(|lhs, rhs| lhs.id.cmp(&rhs.id));

        match serde_json::to_string(&report) {
            Err(error) => {
                return Err(web_view::Error::Custom(Box::new(error)))
            }

            Ok(json) => {
                if let Err(error) = file.write_all(json.as_bytes()) {
                    return Err(web_view::Error::Custom(Box::new(error)))
                }
            }
        }
    }

    Ok(())
}

fn import_markdown<'a>(view: &mut web_view::WebView<'a, ()>, state: &mut StateData) -> web_view::WVResult {
    if let Some(path) = tinyfiledialogs::open_file_dialog("Select a Markdown file", "*.md", None) {
        match std::fs::read_to_string(path) {
            Err(error) => {
                return Err(web_view::Error::Custom(Box::new(error)))
            }

            Ok(md) => {

                let arena = comrak::Arena::new();
                
                let mut options = comrak::ComrakOptions::default();
                options.extension.autolink = true;
                options.extension.description_lists = true;
                options.extension.footnotes = true;
                options.extension.strikethrough = true;
                options.extension.superscript = true;
                options.extension.table = true;
                options.extension.tagfilter = true;
                options.extension.tasklist = true;

                let root = comrak::parse_document(&arena, md.as_str(), &options);
                let mut iter = root.children();
                let mut node = iter.next().unwrap();
                let mut valid = true;

                while valid {
                    use comrak::{
                        nodes::{NodeHeading, NodeValue},
                    };
                    use report::Severity;

                    let mut finding = Finding {
                        id: state.current_finding_id + 1,
                        title: String::new(),
                        class: String::new(),
                        severity: None,
                        location: String::new(),
                        description: String::new(),
                        recommendation: String::new(),
                        alleviation: String::new()
                    };

                    if let NodeValue::Heading(NodeHeading { level: 3, setext: false }) = node.data.borrow().value {
                        for child in node.children() {
                            if let NodeValue::Text(ref text) = child.data.borrow().value {
                                finding.title.push_str(std::str::from_utf8(text).unwrap());
                            }
                        }
                    } else {
                        panic!("Expected heading level 3, found {:?}", node);
                    }

                    node = match iter.next() {
                        None => break,
                        Some(node) => node
                    };

                    if let NodeValue::Table(_) = node.data.borrow().value {
                        for row_node in node.children() {
                            if let NodeValue::TableRow(is_header) = row_node.data.borrow().value {
                                if !is_header {
                                    for (cell_index, cell_node) in row_node.children().enumerate() {
                                        if let NodeValue::TableCell = cell_node.data.borrow().value {
                                            for child in cell_node.children() {
                                                if let NodeValue::Text(ref text) = child.data.borrow().value {
                                                    let text = std::str::from_utf8(text).unwrap();

                                                    match cell_index {
                                                        0 => {
                                                            finding.class.push_str(text);
                                                        }

                                                        1 => {
                                                            finding.severity = match text {
                                                                "Critical" => Some(Severity::Critical),
                                                                "Major" => Some(Severity::Major),
                                                                "Minor" => Some(Severity::Minor),
                                                                "Informational" => Some(Severity::Informational),
                                                                _ => None
                                                            };
                                                        }

                                                        2 => {
                                                            finding.location.push_str(text)
                                                        },
                                                        _ => ()
                                                    }
                                                }
                                            }
                                        } else {
                                            panic!("Expected table cell");
                                        }
                                    }
                                }
                            } else {
                                panic!("Expected table row, found {:?}", row_node);
                            }
                        }
                    }

                    node = match iter.next() {
                        None => break,
                        Some(node) => node
                    };

                    if let NodeValue::Heading(NodeHeading { level: 4, setext: false }) = node.data.borrow().value {
                        let mut heading = String::new();

                        for child in node.children() {
                            if let NodeValue::Text(ref text) = child.data.borrow().value {
                                heading.push_str(std::str::from_utf8(text).unwrap());
                            }
                        }

                        assert!(heading == "Description:", "Expected description heading");
                    } else {
                        panic!("Expected heading level 4, found {:?}", node);
                    }
                        
                    node = match iter.next() {
                        None => break,
                        Some(node) => node
                    };

                    'description: loop {
                        match node.data.borrow().value {
                            NodeValue::Heading(NodeHeading { level: 3, setext: false }) => break 'description,
                            NodeValue::Heading(NodeHeading { level: 4, setext: false }) => break 'description,
                            NodeValue::Text(ref text) => finding.description.push_str(std::str::from_utf8(text).unwrap()),
                            NodeValue::Code(ref text) => finding.description.push_str(format!("`{}`", std::str::from_utf8(text).unwrap()).as_str()),
                            NodeValue::CodeBlock(ref code) => finding.description.push_str(format!("\n\n```\n{}```\n", std::str::from_utf8(code.literal.as_slice()).unwrap()).as_str()),
                            NodeValue::HtmlBlock(_) => (),
                            NodeValue::Paragraph => for paranode in node.children() {
                                match paranode.data.borrow().value {
                                    NodeValue::Heading(NodeHeading { level: 3, setext: false }) => break 'description,
                                    NodeValue::Heading(NodeHeading { level: 4, setext: false }) => break 'description,
                                    NodeValue::Text(ref text) => finding.description.push_str(std::str::from_utf8(text).unwrap()),
                                    NodeValue::Code(ref text) => finding.description.push_str(format!("`{}`", std::str::from_utf8(text).unwrap()).as_str()),
                                    NodeValue::CodeBlock(ref code) => finding.description.push_str(format!("\n\n```\n{}```\n", std::str::from_utf8(code.literal.as_slice()).unwrap()).as_str()),
                                    NodeValue::HtmlBlock(_) => (),
                                    ref node => println!("Unused description paragraph node: {:?}", node)
                                }
                            }
                            ref node => println!("Unused description node: {:?}", node)
                        }
                        
                        node = match iter.next() {
                            None => break,
                            Some(node) => node
                        };
                    }

                    if let NodeValue::Heading(NodeHeading { level: 4, setext: false }) = node.data.borrow().value {
                        let mut heading = String::new();

                        for child in node.children() {
                            if let NodeValue::Text(ref text) = child.data.borrow().value {
                                heading.push_str(std::str::from_utf8(text).unwrap());
                            }
                        }

                        assert!(heading == "Recommendation:", "Expected recommendation heading");
                    } else {
                        panic!("Expected heading level 4, found {:?}", node);
                    }

                    node = match iter.next() {
                        None => break,
                        Some(node) => node
                    };

                    'recommendation: loop {
                        match node.data.borrow().value {
                            NodeValue::Heading(NodeHeading { level: 3, setext: false }) => break 'recommendation,
                            NodeValue::Heading(NodeHeading { level: 4, setext: false }) => break 'recommendation,
                            NodeValue::Text(ref text) => finding.recommendation.push_str(std::str::from_utf8(text).unwrap()),
                            NodeValue::Code(ref text) => finding.recommendation.push_str(format!("`{}`", std::str::from_utf8(text).unwrap()).as_str()),
                            NodeValue::CodeBlock(ref code) => finding.recommendation.push_str(format!("\n\n```\n{}```\n", std::str::from_utf8(code.literal.as_slice()).unwrap()).as_str()),
                            NodeValue::HtmlBlock(_) => (),
                            NodeValue::Paragraph => for paranode in node.children() {
                                match paranode.data.borrow().value {
                                    NodeValue::Heading(NodeHeading { level: 3, setext: false }) => break 'recommendation,
                                    NodeValue::Heading(NodeHeading { level: 4, setext: false }) => break 'recommendation,
                                    NodeValue::Text(ref text) => finding.recommendation.push_str(std::str::from_utf8(text).unwrap()),
                                    NodeValue::Code(ref text) => finding.recommendation.push_str(format!("`{}`", std::str::from_utf8(text).unwrap()).as_str()),
                                    NodeValue::CodeBlock(ref code) => finding.recommendation.push_str(format!("\n\n```\n{}```\n", std::str::from_utf8(code.literal.as_slice()).unwrap()).as_str()),
                                    NodeValue::HtmlBlock(_) => (),
                                    ref node => println!("Unused recommendation paragraph node: {:?}", node)
                                }
                            }
                            ref node => println!("Unused recommendation node: {:?}", node)
                        }

                        node = match iter.next() {
                            None => {
                                valid = false;
                                break;
                            },
                            Some(node) => node
                        };
                    }

                    assert!(state.findings.insert(finding.id, finding.clone()).is_none());
                    state.current_finding_id += 1;

                    add_finding_to_web_view(view, &finding)?;
                }
            }
        }
    }

    Ok(())
}

fn export_markdown<'a>(state: &mut StateData) -> web_view::WVResult {
    if let Some(path) = tinyfiledialogs::save_file_dialog("Select a Markdown file", "workbook.md") {
        use std::{fs::File, io::Write};

        let mut file = match File::create(path) {
            Err(error) => return Err(web_view::Error::Custom(Box::new(error))),
            Ok(file) => file
        };

        let mut findings = vec![];

        for finding in state.findings.values() {
            findings.push(finding.clone());
        }

        findings.sort_by(|lhs, rhs| lhs.id.cmp(&rhs.id));

        let mut md = String::new();
        md.push_str("| ID | Title | Type | Severity |\n");
        md.push_str("|-:|-|-|-|\n");

        for finding in &findings {
            let severity = match finding.severity.unwrap() {
                report::Severity::Critical => "Critical",
                report::Severity::Major => "Major",
                report::Severity::Minor => "Minor",
                report::Severity::Informational => "Informational"
            };

            md.push_str(
                format!(
                    "| <span class=\"{sev_low}\">[XXX-{id:02}](#XXX-{id:02})</span> | <span class=\"{sev_low}\">{title}</span> | <span class=\"{sev_low}\">{kind}</span> | <span class=\"{sev_low}\">{sev}</span> |\n",
                    sev_low = severity.to_lowercase(), sev = severity, id = finding.id, title = finding.title, kind = finding.class
                ).as_str()
            );
        }

        md.push_str("\n");

        for finding in &findings {
            let severity = match finding.severity.unwrap() {
                report::Severity::Critical => "Critical",
                report::Severity::Major => "Major",
                report::Severity::Minor => "Minor",
                report::Severity::Informational => "Informational"
            };

            let severity_low = severity.to_lowercase();

            md.push_str("\n");
            md.push_str("<div style=\"page-break-after: always\"></div>\n");
            md.push_str("\n");
            md.push_str(format!("### ![](https://svgshare.com/i/QKR.svg)XXX-{:02}: {}\n", finding.id, finding.title).as_str());
            md.push_str("\n");
            md.push_str("| Type | Severity | Location |\n");
            md.push_str("|-|-|-|\n");
            md.push_str(format!("| {} | {} | <span class=\"{}\">{}</span> |\n", finding.class, severity, severity.to_lowercase(), finding.location).as_str());
            md.push_str("\n");
            md.push_str(format!("#### <span class=\"{}\">Description:</span>\n", severity_low).as_str());
            md.push_str("\n");
            md.push_str(format!("<div class=\"{}\">\n", severity_low).as_str());
            md.push_str("\n");
            md.push_str(format!("{}\n", finding.description.as_str()).as_str());
            md.push_str("\n");
            md.push_str("</div>\n");
            md.push_str("\n");
            md.push_str(format!("#### <span class=\"{}\">Recommendation:</span>\n", severity_low).as_str());
            md.push_str("\n");
            md.push_str(format!("<div class=\"{}\">\n", severity_low).as_str());
            md.push_str("\n");
            md.push_str(format!("{}\n", finding.recommendation.as_str()).as_str());
            md.push_str("\n");
            md.push_str("</div>\n");
            md.push_str("\n");
        }

        if let Err(error) = file.write_all(md.as_bytes()) {
            return Err(web_view::Error::Custom(Box::new(error)))
        }
    }

    Ok(())
}

fn export_pdf<'a>(view: &mut web_view::WebView<'a, ()>) -> web_view::WVResult {
    view.eval("alert('PDF exporting is not currently supported')")
}

fn create_checklist_entry<'a>(view: &mut web_view::WebView<'a, ()>, state: &mut StateData) -> web_view::WVResult {
    let id = state.checklist.len();
    state.checklist.push((false, String::new()));
    add_checklist_entry_to_web_view(view, id, (false, ""))
}

fn remove_checklist_entry<'a>(view: &mut web_view::WebView<'a, ()>, state: &mut StateData, id: usize) -> web_view::WVResult {
    if id < state.checklist.len() {
        let _ = state.checklist.remove(id);

        let mut entry_table = HtmlElement::get(format!("checklist{}_table", id).as_str());
        entry_table.remove();
        entry_table.build(view)
    } else {
        Err(web_view::Error::Custom(Box::new(format!("No checklist entry for id {} was found!", id))))
    }
}

fn clear_checklist_entries<'a>(view: &mut web_view::WebView<'a, ()>, state: &mut StateData) -> web_view::WVResult {
    let mut entries = vec![];

    for entry in &state.checklist {
        entries.push(entry.clone());
    }

    for (id, _) in entries.iter().enumerate() {
        let _ = remove_checklist_entry(view, state, id);
    }

    Ok(())
}

fn set_checklist_entry_checked<'a>(view: &mut web_view::WebView<'a, ()>, state: &mut StateData, id: usize, checked: bool) -> web_view::WVResult {
    if id < state.checklist.len() {
        state.checklist[id].0 = checked;

        let mut entry_check_input = HtmlElement::get(format!("checklist{}_check_input", id).as_str());
        entry_check_input.set_checked(checked);
        entry_check_input.build(view)
    } else {
        Err(web_view::Error::Custom(Box::new(format!("No checklist entry for id {} was found!", id))))
    }
}

fn set_checklist_entry_text<'a>(view: &mut web_view::WebView<'a, ()>, state: &mut StateData, id: usize, text: &str) -> web_view::WVResult {
    if id < state.checklist.len() {
        state.checklist[id].1 = text.to_string();

        let mut entry_text_input = HtmlElement::get(format!("checklist{}_text_input", id).as_str());
        entry_text_input.set_value(text);
        entry_text_input.build(view)
    
    } else {
        Err(web_view::Error::Custom(Box::new(format!("No checklist entry for id {} was found!", id))))
    }
}

fn create_finding<'a>(view: &mut web_view::WebView<'a, ()>, state: &mut StateData) -> web_view::WVResult {
    state.current_finding_id += 1;

    let finding = Finding {
        id: state.current_finding_id,
        title: format!("Finding{}", state.current_finding_id),
        class: String::new(),
        severity: None,
        location: String::new(),
        description: String::new(),
        recommendation: String::new(),
        alleviation: String::new()
    };

    // Attempt to add the new finding to the state data findings map
    assert!(state.findings.insert(finding.id, finding.clone()).is_none());

    // Attempt to add the new finding to the web view
    add_finding_to_web_view(view, &finding)
}

fn copy_finding<'a>(view: &mut web_view::WebView<'a, ()>, state: &mut StateData, id: usize) -> web_view::WVResult {
    if let Some(finding) = state.findings.get(&id) {
        // Set the copied finding in the state data
        state.copied_finding = Some(finding.clone());

        // Attempt to enable the paste button in the web view
        let mut paste_button = html::HtmlElement::get("paste_button");
        paste_button.set_disabled(false);
        paste_button.build(view)
    } else {
        Err(web_view::Error::Custom(Box::new(format!("No finding for id {} was found!", id))))
    }
}

fn paste_finding<'a>(view: &mut web_view::WebView<'a, ()>, state: &mut StateData) -> web_view::WVResult {
    let mut finding = state.copied_finding.clone().unwrap();
    state.current_finding_id += 1;
    finding.id = state.current_finding_id;
    add_finding_to_web_view(view, &finding)?;
    assert!(state.findings.insert(finding.id, finding).is_none());
    Ok(())
}

fn remove_finding<'a>(view: &mut web_view::WebView<'a, ()>, state: &mut StateData, id: usize) -> web_view::WVResult {
    if state.findings.remove(&id).is_some() {
        let _ = state.findings.remove(&id);

        let mut finding = HtmlElement::get(format!("finding{}", id).as_str());
        finding.remove();
        finding.build(view)?;

        let mut link = HtmlElement::get(format!("finding{}_link_p", id).as_str());
        link.remove();
        link.build(view)
    } else {
        Err(web_view::Error::Custom(Box::new(format!("No finding for id {} was found!", id))))
    }
}

fn clear_findings<'a>(view: &mut web_view::WebView<'a, ()>, state: &mut StateData) -> web_view::WVResult {
    let mut ids = vec![];

    for id in state.findings.keys() {
        ids.push(*id);
    }

    for id in ids {
        remove_finding(view, state, id)?;
    }

    state.current_finding_id = 0;

    Ok(())
}

fn set_finding_title<'a>(view: &mut web_view::WebView<'a, ()>, state: &mut StateData, id: usize, title: &str) -> web_view::WVResult {
    if let Some(entry) = state.findings.get_mut(&id) {
        entry.title = title.to_string();

        let mut finding_link = HtmlElement::get(format!("finding{}_link", id).as_str());
        finding_link.set_inner_html(title);
        finding_link.build(view)?;

        let mut finding_title_id = HtmlElement::get(format!("finding{}_title_id", id).as_str());
        finding_title_id.set_inner_html(format!("{}: ", id).as_str());
        finding_title_id.build(view)?;

        let mut finding_title = HtmlElement::get(format!("finding{}_title", id).as_str());
        finding_title.set_inner_html(title);
        finding_title.build(view)
    } else {
        Err(web_view::Error::Custom(Box::new(format!("No finding for id {} was found!", id))))
    }
}

fn set_finding_type<'a>(view: &mut web_view::WebView<'a, ()>, state: &mut StateData, id: usize, class: &str) -> web_view::WVResult {
    if let Some(entry) = state.findings.get_mut(&id) {
        entry.class = class.to_string();

        let mut finding_type = HtmlElement::get(format!("finding{}_type", id).as_str());
        finding_type.set_inner_html(class);
        finding_type.build(view)
    } else {
        Err(web_view::Error::Custom(Box::new(format!("No finding for id {} was found!", id))))
    }
}

fn set_finding_severity<'a>(view: &mut web_view::WebView<'a, ()>, state: &mut StateData, id: usize, severity: &str) -> web_view::WVResult {
    if let Some(entry) = state.findings.get_mut(&id) {
        entry.severity = match severity {
            "critical" => Some(report::Severity::Critical),
            "major" => Some(report::Severity::Major),
            "minor" => Some(report::Severity::Minor),
            "informational" => Some(report::Severity::Informational),
            _ => None
        };

        let style = match entry.severity {
            Some(report::Severity::Critical) => "color: red",
            Some(report::Severity::Major) => "color: orange",
            Some(report::Severity::Minor) => "color: yellow",
            Some(report::Severity::Informational) => "color: green",
            None => "color: inherit",
        };

        let mut finding_link = HtmlElement::get(format!("finding{}_link", id).as_str());
        finding_link.set_attribute("style", style);
        finding_link.build(view)?;

        let mut description_header = HtmlElement::get(format!("finding{}_description_header", id).as_str());
        description_header.set_attribute("style", style);
        description_header.build(view)?;

        let mut recommendation_header = HtmlElement::get(format!("finding{}_recommendation_header", id).as_str());
        recommendation_header.set_attribute("style", style);
        recommendation_header.build(view)?;

        let mut alleviation_header = HtmlElement::get(format!("finding{}_alleviation_header", id).as_str());
        alleviation_header.set_attribute("style", style);
        alleviation_header.build(view)?;

        let mut option = HtmlElement::get(format!("finding{}_severity_{}_option", id, severity).as_str());
        option.set_selected(true);
        option.build(view)
    } else {
        Err(web_view::Error::Custom(Box::new(format!("No finding for id {} was found!", id))))
    }
}

fn set_finding_location<'a>(view: &mut web_view::WebView<'a, ()>, state: &mut StateData, id: usize, location: &str) -> web_view::WVResult {
    if let Some(entry) = state.findings.get_mut(&id) {
        entry.location = location.to_string();

        let mut finding_location = HtmlElement::get(format!("finding{}_location", id).as_str());
        finding_location.set_inner_html(location);
        finding_location.build(view)
    } else {
        Err(web_view::Error::Custom(Box::new(format!("No finding for id {} was found!", id))))
    }
}

fn set_finding_description<'a>(view: &mut web_view::WebView<'a, ()>, state: &mut StateData, id: usize, description: &str) -> web_view::WVResult {
    if let Some(entry) = state.findings.get_mut(&id) {
        entry.description = description.to_string();

        let mut finding_description = HtmlElement::get(format!("finding{}_description", id).as_str());
        finding_description.set_inner_html(description);
        finding_description.build(view)
    } else {
        Err(web_view::Error::Custom(Box::new(format!("No finding for id {} was found!", id))))
    }
}

fn set_finding_recommendation<'a>(view: &mut web_view::WebView<'a, ()>, state: &mut StateData, id: usize, recommendation: &str) -> web_view::WVResult {
    if let Some(entry) = state.findings.get_mut(&id) {
        entry.recommendation = recommendation.to_string();

        let mut finding_recommendation = HtmlElement::get(format!("finding{}_recommendation", id).as_str());
        finding_recommendation.set_inner_html(recommendation);
        finding_recommendation.build(view)
    } else {
        Err(web_view::Error::Custom(Box::new(format!("No finding for id {} was found!", id))))
    }
}

fn set_finding_alleviation<'a>(view: &mut web_view::WebView<'a, ()>, state: &mut StateData, id: usize, alleviation: &str) -> web_view::WVResult {
    if let Some(entry) = state.findings.get_mut(&id) {
        entry.alleviation = alleviation.to_string();

        let mut finding_alleviation = HtmlElement::get(format!("finding{}_alleviation", id).as_str());
        finding_alleviation.set_inner_html(alleviation);
        finding_alleviation.build(view)
    } else {
        Err(web_view::Error::Custom(Box::new(format!("No finding for id {} was found!", id))))
    }
}

fn add_finding_to_web_view<'a>(view: &mut web_view::WebView<'a, ()>, finding: &Finding) -> web_view::WVResult {
    //
    // Create a new finding table
    //

    let mut new_table = HtmlElement::new("table", "new_table");
    new_table.set_attribute("id", format!("finding{}", finding.id).as_str());
    new_table.set_attribute("style", "margin: 1rem");

    let new_row = new_table.insert_row(0, "new_row");
    let new_cell = new_row.insert_cell(0, "new_cell");

    //
    // Create the toolbar table for the new finding
    //

    let mut toolbar_table = HtmlElement::new("table", "toolbar_table");
    toolbar_table.set_attribute("style", "table-layout: fixed; width: 100%");

    let toolbar_row = toolbar_table.insert_row(0, "toolbar_row");
    
    let toolbar_cell = toolbar_row.insert_cell(0, "toolbar_cell");

    // ---------------------------------------------------

    let mut toolbar_close_button = HtmlElement::new("button", "toolbar_close_button");
    toolbar_close_button.set_inner_html("X");
    toolbar_close_button.set_field("style.backgroundColor", "tomato");
    toolbar_close_button.set_attribute("onclick", format!("external.invoke(\"remove_finding {}\")", finding.id).as_str());
    
    toolbar_cell.append_child(toolbar_close_button);

    // ---------------------------------------------------

    let mut toolbar_copy_button = HtmlElement::new("button", "toolbar_copy_button");
    toolbar_copy_button.set_attribute("onclick", format!("external.invoke(\"copy_finding {}\")", finding.id).as_str());
    toolbar_copy_button.set_inner_html("Copy");

    toolbar_cell.append_child(toolbar_copy_button);

    // ---------------------------------------------------

    new_cell.append_child(toolbar_table);

    //
    // Create the title table for the new finding
    //

    let mut title_table = HtmlElement::new("table", "title_table");
    title_table.set_attribute("style", "width: 100%");

    let title_row = title_table.insert_row(0, "title_row");

    // ---------------------------------------------------

    let title_image_cell = title_row.insert_cell(0, "title_image_cell");
    title_image_cell.set_attribute("style", "vertical-align: middle");
    
    let mut title_image = HtmlElement::new("img", "title_image");
    title_image.set_attribute("src", "https://i.imgur.com/QwKomxS.png");
    title_image.set_attribute("style", "width: 40px; height: auto");
    
    title_image_cell.append_child(title_image);

    // ---------------------------------------------------

    let title_id_cell = title_row.insert_cell(1, "title_id_cell");
    title_id_cell.set_inner_html(format!("{}: ", finding.id).as_str());
    title_id_cell.set_attribute("id", format!("finding{}_title_id", finding.id).as_str());
    title_id_cell.set_attribute("style", "font-size: 1.5rem; padding-left: 0.25rem; padding-right: 0.25rem; text-align: center; vertical-align: middle");
    
    // ---------------------------------------------------

    let title_text_cell = title_row.insert_cell(2, "title_text_cell");
    title_text_cell.set_attribute("style", "width: 100%; vertical-align: middle");
    
    let mut title_text_input = HtmlElement::new("input", "title_text_input");
    title_text_input.set_attribute("type", "text");
    title_text_input.set_attribute("id", format!("finding{}_title", finding.id).as_str());
    title_text_input.set_attribute("value", finding.title.as_str());
    title_text_input.set_attribute("style", "font-size: 1.5rem");
    title_text_input.set_attribute("onchange", format!("external.invoke('set_finding_{} {} \"' + this.value.toString().replaceAll('\"', '\\\\'\\\\'') + '\"')", "title", finding.id).as_str());
    
    title_text_cell.append_child(title_text_input);
    
    // ---------------------------------------------------

    new_cell.append_child(title_table);

    //
    // Create the header table for the new finding
    //

    let mut header_table = HtmlElement::new("table", "header_table");
    header_table.set_attribute("style", "width: 100%");

    // ---------------------------------------------------

    let header_row1 = header_table.insert_row(0, "header_row1");

    let mut create_header_cell = |index, text| {
        let header_cell = header_row1.insert_cell(index, format!("header_cell{}", index).as_str());
        header_cell.set_attribute("style", "padding-top: 6.5px; padding-bottom: 6.5px");
        header_cell.set_inner_html(text);
    };

    create_header_cell(0, "Type");
    create_header_cell(1, "Severity");
    create_header_cell(2, "Location");

    // ---------------------------------------------------

    let header_row2 = header_table.insert_row(1, "header_row2");

    let header_type_cell = header_row2.insert_cell(0, "header_type_cell");

    let mut header_type_input = HtmlElement::new("input", "header_type_input");
    header_type_input.set_attribute("type", "text");
    header_type_input.set_attribute("value", finding.class.as_str());
    header_type_input.set_attribute("id", format!("finding{}_type", finding.id).as_str());
    header_type_input.set_attribute("onchange", format!("external.invoke('set_finding_{} {} \"' + this.value.toString().replaceAll('\"', '\\\\'\\\\'') + '\"')", "type", finding.id).as_str());
    
    header_type_cell.append_child(header_type_input);

    // ---------------------------------------------------

    let header_severity_cell = header_row2.insert_cell(1, "header_severity_cell");

    let mut header_severity_select = HtmlElement::new("select", "header_severity_select");
    header_severity_select.set_attribute("id", format!("finding{}_severity", finding.id).as_str());
    header_severity_select.set_attribute("onchange", format!("external.invoke('set_finding_{} {} \"' + this.value.toString().replaceAll('\"', '\\\\'\\\\'') + '\"')", "severity", finding.id).as_str());

    let mut create_severity_option = |name, text| {
        let mut option = HtmlElement::new("option", format!("finding{}_severity_{}_option", finding.id, name).as_str());
        option.set_attribute("value", name);
        option.set_inner_html(text);

        if let Some(severity) = finding.severity {
            match severity {
                report::Severity::Critical => if name == "critical" { option.set_selected(true) },
                report::Severity::Major => if name == "major" { option.set_selected(true) },
                report::Severity::Minor => if name == "minor" { option.set_selected(true) },
                report::Severity::Informational => if name == "informational" { option.set_selected(true) }
            }
        }

        header_severity_select.append_child(option);
    };

    create_severity_option("none", "Select...");
    create_severity_option("critical", "Critical");
    create_severity_option("major", "Major");
    create_severity_option("minor", "Minor");
    create_severity_option("informational", "Informational");

    header_severity_cell.append_child(header_severity_select);
    
    // ---------------------------------------------------

    let header_location_cell = header_row2.insert_cell(2, "header_location_cell");

    let mut header_location_input = HtmlElement::new("input", "header_location_input");
    header_location_input.set_attribute("type", "text");
    header_location_input.set_attribute("value", finding.location.as_str());
    header_location_input.set_attribute("id", format!("finding{}_location", finding.id).as_str());
    header_location_input.set_attribute("onchange", format!("external.invoke('set_finding_{} {} \"' + this.value.toString().replaceAll('\"', '\\\\'\\\\'') + '\"')", "location", finding.id).as_str());

    header_location_cell.append_child(header_location_input);

    //
    // Done building the header table for the new finding
    //

    new_cell.append_child(header_table);

    //
    // Create the text areas for the new finding
    //

    let severity_style = match finding.severity {
        Some(report::Severity::Critical) => "color: red",
        Some(report::Severity::Major) => "color: orange",
        Some(report::Severity::Minor) => "color: yellow",
        Some(report::Severity::Informational) => "color: green",
        None => "color: inherit",
    };

    let mut create_text_area = |name, text, string| {
        let mut header = HtmlElement::new("h4", format!("finding{}_{}_header", finding.id, name).as_str());
        header.set_inner_html(text);
        header.set_attribute("style", severity_style);
        header.set_attribute("id", format!("finding{}_{}_header", finding.id, name).as_str());
        new_cell.append_child(header);
    
        let mut textarea = HtmlElement::new("textarea", format!("finding{}_{}_textarea", finding.id, name).as_str());
        textarea.set_inner_html(string);
        textarea.set_attribute("rows", "4");
        textarea.set_attribute("cols", "80");
        textarea.set_attribute("maxlength", "9999");
        textarea.set_field("style.resize", "vertical");
        textarea.set_attribute("id", format!("finding{}_{}", finding.id, name).as_str());
        textarea.set_attribute(
            "onchange",
            format!("external.invoke('set_finding_{} {} \"' + this.value.toString().replaceAll('\"', '\\\\'\\\\'') + '\"')", name, finding.id).as_str()
        );

        new_cell.append_child(textarea);
    };

    create_text_area("description", "Description:", finding.description.as_str());
    create_text_area("recommendation", "Recommendation:", finding.recommendation.as_str());
    //create_text_area("alleviation", "Alleviation:", finding.alleviation.as_str());

    new_cell.append_child(HtmlElement::new("p", "spacer"));

    //
    // Done building new finding table
    //

    let mut findings = HtmlElement::get("findings");
    findings.append_child(new_table);

    findings.build(view)?;

    // Jump to the new finding
    view.eval(format!("window.location = '#finding{}'", finding.id).as_str()).unwrap();

    //
    // Create the table of contents link
    //

    let mut toc_findings = HtmlElement::get("toc_findings");
    
    let mut p = HtmlElement::new("p", "p");
    p.set_attribute("id", format!("finding{}_link_p", finding.id).as_str());

    let mut link_id = HtmlElement::new("span", "link_id");
    link_id.set_inner_html(format!("{}: ", finding.id).as_str());
    p.append_child(link_id);

    let mut link = HtmlElement::new("a", "link");
    link.set_attribute("style", severity_style);
    link.set_attribute("id", format!("finding{}_link", finding.id).as_str());
    link.set_attribute("href", format!("#finding{}", finding.id).as_str());
    link.set_inner_html(finding.title.as_str());
    p.append_child(link);

    toc_findings.append_child(p);
    toc_findings.build(view)
}

fn add_checklist_entry_to_web_view<'a>(view: &mut web_view::WebView<'a, ()>, id: usize, entry: (bool, &str)) -> web_view::WVResult {
    let mut entry_check_input = HtmlElement::new("input", "entry_check_input");
    entry_check_input.set_attribute("onclick", format!("external.invoke('set_checklist_entry_checked {} ' + this.checked.toString())", id).as_str());
    entry_check_input.set_attribute("id", format!("checklist{}_check_input", id).as_str());
    entry_check_input.set_attribute("type", "checkbox");
    entry_check_input.set_checked(entry.0);
    
    let mut entry_text_input = HtmlElement::new("input", "entry_text_input");
    entry_text_input.set_attribute("type", "text");
    entry_text_input.set_attribute("id", format!("checklist{}_text_input", id).as_str());
    entry_text_input.set_attribute("style", "line-height: 1rem; margin: 0%; padding: 0%");
    entry_text_input.set_attribute("onchange", format!("external.invoke('set_checklist_entry_text {} \"' + this.value.toString().replaceAll('\"', '\\\\'\\\\'') + '\"')", id).as_str());
    entry_text_input.set_value(entry.1);

    let mut entry_close_button = HtmlElement::new("button", "entry_close_button");
    entry_close_button.set_inner_html("X");
    entry_close_button.set_attribute("style", "background-color: tomato; width: 100%; height: 100%");
    entry_close_button.set_attribute("onclick", format!("external.invoke(\"remove_checklist_entry {}\")", id).as_str());
    
    let mut entry_table = HtmlElement::new("table", "entry_table");
    entry_table.set_attribute("id", format!("checklist{}_table", id).as_str());
    entry_table.set_attribute("style", "width: 100%");

    let entry_row = entry_table.insert_row(0, "entry_row");
    
    let entry_check_cell = entry_row.insert_cell(0, "entry_check_cell");
    entry_check_cell.append_child(entry_check_input);

    let entry_text_cell = entry_row.insert_cell(1, "entry_text_cell");
    entry_text_cell.set_attribute("style", "width: 100%");
    entry_text_cell.append_child(entry_text_input);

    let entry_close_cell = entry_row.insert_cell(2, "entry_close_cell");
    entry_close_cell.append_child(entry_close_button);

    let mut toc_checklist = HtmlElement::get("toc_checklist");
    toc_checklist.append_child(entry_table);
    toc_checklist.build(view)
}
