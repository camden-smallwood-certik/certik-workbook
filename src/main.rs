#[macro_use]
extern crate serde_derive;

pub mod command;
pub mod html;
pub mod markdown;
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
    pub findings: HashMap<usize, Finding>,
    pub copied_finding: Option<Finding>
}

impl StateData {
    pub fn new() -> Self {
        Self {
            initialized: false,
            current_finding_id: 0,
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

                    Ok(report::Report { mut findings, .. }) => {
                        clear_findings(view, state)?;
                        
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
            overview: "Executive Overview".to_string(),
            findings: vec![]
        };

        for (_, finding) in &state.findings {
            report.findings.push(finding.clone());
        }

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
                let root = comrak::parse_document(&arena, md.as_str(), &comrak::ComrakOptions::default());
                
                let mut parser = markdown::MarkdownParser::new();
                parser.parse_ast_node(state, root);

                for ref finding in parser.findings {
                    assert!(state.findings.insert(finding.id, finding.clone()).is_none());
                    add_finding_to_web_view(view, finding)?;
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

        let mut md = String::new();

        for finding in state.findings.values() {
            let severity = match finding.severity.unwrap() {
                report::Severity::Critical => "Critical",
                report::Severity::Major => "Major",
                report::Severity::Minor => "Minor",
                report::Severity::Informational => "Informational"
            };

            md.push_str("---\n");
            md.push_str("\n");
            md.push_str(format!("<section id=\"{}\">\n", severity.to_lowercase()).as_str());
            md.push_str("\n");
            md.push_str(format!("### ![](https://svgshare.com/i/QKR.svg){}\n", finding.title).as_str());
            md.push_str("\n");
            md.push_str("| Type | Severity | Location | Status |\n");
            md.push_str("|-|-|-|-|\n");
            md.push_str(format!("| {} | {} | {} | |\n", finding.class, severity, finding.location).as_str());
            md.push_str("\n");
            md.push_str("#### Description:\n");
            md.push_str("\n");
            md.push_str(format!("{}\n", finding.description.as_str()).as_str());
            md.push_str("\n");
            md.push_str("#### Recommendation:\n");
            md.push_str("\n");
            md.push_str(format!("{}\n", finding.recommendation.as_str()).as_str());
            md.push_str("\n");
            md.push_str("#### Alleviation:\n");
            md.push_str("\n");
            md.push_str(format!("{}\n", finding.alleviation.as_str()).as_str());
            md.push_str("\n");
            md.push_str("</section>\n");
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
            Some(report::Severity::Critical) => "color: rgb(230, 68, 60)",
            Some(report::Severity::Major) => "color: rgb(249, 159, 28)",
            Some(report::Severity::Minor) => "color: rgb(65, 91, 169)",
            Some(report::Severity::Informational) => "color: rgb(85, 155, 74)",
            None => "color: rgb(85, 84, 85)",
        };

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
    toolbar_close_button.set_attribute("onclick", format!("external.invoke(\"remove_finding {}\")", finding.id).as_str());
    toolbar_close_button.set_inner_html("X");
    
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
    title_image.set_attribute("src", "https://svgshare.com/i/QKR.svg");
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
    title_text_input.set_attribute("onchange", format!("set_finding_value(\"title\", {})", finding.id).as_str());
    
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
    header_type_input.set_attribute("onchange", format!("set_finding_value(\"type\", {})", finding.id).as_str());
    
    header_type_cell.append_child(header_type_input);

    // ---------------------------------------------------

    let header_severity_cell = header_row2.insert_cell(1, "header_severity_cell");

    let mut header_severity_select = HtmlElement::new("select", "header_severity_select");
    header_severity_select.set_attribute("id", format!("finding{}_severity", finding.id).as_str());
    header_severity_select.set_attribute("onchange", format!("set_finding_value(\"severity\", {})", finding.id).as_str());

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
    header_location_input.set_attribute("onchange", format!("set_finding_value(\"location\", {})", finding.id).as_str());

    header_location_cell.append_child(header_location_input);

    //
    // Done building the header table for the new finding
    //

    new_cell.append_child(header_table);

    //
    // Create the text areas for the new finding
    //

    let header_style = match finding.severity {
        Some(report::Severity::Critical) => "color: rgb(230, 68, 60)",
        Some(report::Severity::Major) => "color: rgb(249, 159, 28)",
        Some(report::Severity::Minor) => "color: rgb(65, 91, 169)",
        Some(report::Severity::Informational) => "color: rgb(85, 155, 74)",
        None => "color: rgb(85, 84, 85)",
    };

    let mut create_text_area = |name, text, string| {
        let mut header = HtmlElement::new("h4", format!("finding{}_{}_header", finding.id, name).as_str());
        header.set_inner_html(text);
        header.set_attribute("style", header_style);
        header.set_attribute("id", format!("finding{}_{}_header", finding.id, name).as_str());
        new_cell.append_child(header);
    
        let mut textarea = HtmlElement::new("textarea", format!("finding{}_{}_textarea", finding.id, name).as_str());
        textarea.set_inner_html(string);
        textarea.set_attribute("rows", "4");
        textarea.set_attribute("cols", "80");
        textarea.set_attribute("maxlength", "9999");
        textarea.set_field("style.resize", "vertical");
        textarea.set_attribute("id", format!("finding{}_{}", finding.id, name).as_str());
        textarea.set_attribute("onchange", format!("set_finding_value(\"{}\", {})", name, finding.id).as_str());
        new_cell.append_child(textarea);
    };

    create_text_area("description", "Description:", finding.description.as_str());
    create_text_area("recommendation", "Recommendation:", finding.recommendation.as_str());
    create_text_area("alleviation", "Alleviation:", finding.alleviation.as_str());

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
    link.set_attribute("id", format!("finding{}_link", finding.id).as_str());
    link.set_attribute("href", format!("#finding{}", finding.id).as_str());
    link.set_inner_html(finding.title.as_str());
    p.append_child(link);

    toc_findings.append_child(p);
    toc_findings.build(view)
}
