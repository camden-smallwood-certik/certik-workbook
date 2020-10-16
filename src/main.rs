#[macro_use]
extern crate serde_derive;

pub mod html;
pub mod report;

use crate::{
    html::HtmlElement,
    report::Finding
};

use std::collections::HashMap;

#[derive(Debug)]
pub struct StateData {
    pub current_finding_id: usize,
    pub findings: HashMap<usize, Finding>
}

fn main() {
    //
    // TODO: load workbook file here
    //

    let html = include_str!("../html/index.html");

    web_view::builder()
        .title("CertiK Workbook")
        .content(web_view::Content::Html(html))
        .size(1024, 640)
        .resizable(true)
        .debug(true)
        .user_data(
            StateData {
                current_finding_id: 0,
                findings: HashMap::new()
            }
        )
        .invoke_handler(|view, arg| {
            println!("Command: {}", arg);
            if arg.contains(' ') {
                // Tokenize the input command and parameters
                let mut is_string = false;
                let mut tokens = vec![];
                let mut token = String::new();

                // Create an iterator to loop over each character
                let mut iter = arg.chars();

                // Build each token using the character iterator
                while let Some(c) = iter.next() {
                    if c.is_whitespace() {
                        if is_string {
                            // Push all whitespace characters inside a string to the token
                            token.push(c);
                        } else if token.len() != 0 {
                            // Push the current token to the vector if token is not empty
                            tokens.push(token);
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
                    tokens.push(token);
                }

                // Create an iterator to loop over the tokens
                let mut iter = tokens.iter();

                // Handle the input command
                match iter.next().unwrap().as_str() {
                    "remove_finding" => remove_finding(view, iter.next().unwrap().parse().unwrap())?,
                    "set_finding_title" => set_finding_title(view, iter.next().unwrap().parse().unwrap(), iter.next().unwrap())?,
                    "set_finding_type" => set_finding_type(view, iter.next().unwrap().parse().unwrap(), iter.next().unwrap())?,
                    "set_finding_severity" => set_finding_severity(view, iter.next().unwrap().parse().unwrap(), iter.next().unwrap())?,
                    "set_finding_location" => set_finding_location(view, iter.next().unwrap().parse().unwrap(), iter.next().unwrap())?,
                    "set_finding_description" => set_finding_description(view, iter.next().unwrap().parse().unwrap(), iter.next().unwrap())?,
                    "set_finding_recommendation" => set_finding_recommendation(view, iter.next().unwrap().parse().unwrap(), iter.next().unwrap())?,
                    "set_finding_alleviation" => set_finding_alleviation(view, iter.next().unwrap().parse().unwrap(), iter.next().unwrap())?,
                    command => view.eval(format!("alert(\"Command not implemented: '{}'\")", command).as_str())?
                }

                // Verify all parameters were used
                assert!(iter.next().is_none());
            } else {
                // Handle the input command
                match arg {
                    "create_finding" => create_finding(view)?,
                    "clear_findings" => clear_findings(view)?,
                    "export_markdown" => export_markdown(view)?,
                    "load_workbook" => load_workbook(view)?,
                    "save_workbook" => save_workbook(view)?,
                    command => view.eval(format!("alert(\"Command not implemented: \\\"{}\\\"\")", command).as_str())?
                }
            }

            Ok(())
        })
        .run()
        .unwrap();
}

fn create_finding<'a>(view: &mut web_view::WebView<'a, StateData>) -> web_view::WVResult {
    let state = view.user_data_mut();
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

    add_finding(view, &finding)
}

fn clear_findings<'a>(view: &mut web_view::WebView<'a, StateData>) -> web_view::WVResult {
    let mut ids = vec![];

    for id in view.user_data().findings.keys() {
        ids.push(*id);
    }

    for id in ids {
        remove_finding(view, id)?;
    }

    view.user_data_mut().current_finding_id = 0;

    Ok(())
}

fn export_markdown<'a>(view: &mut web_view::WebView<'a, StateData>) -> web_view::WVResult {
    if let Some(path) = tinyfiledialogs::save_file_dialog("Select a Markdown file", "workbook.md") {
        use std::{fs::File, io::Write};

        let mut file = match File::create(path) {
            Err(error) => return Err(web_view::Error::Custom(Box::new(error))),
            Ok(file) => file
        };

        let mut md = String::new();

        for finding in view.user_data().findings.values() {
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

fn load_workbook<'a>(view: &mut web_view::WebView<'a, StateData>) -> web_view::WVResult {
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
                        clear_findings(view)?;
                        
                        findings.sort_by(|lhs, rhs| lhs.id.cmp(&rhs.id));

                        view.user_data_mut().current_finding_id = findings.len();
                        
                        for (_, finding) in findings.iter().enumerate() {
                            add_finding(view, &finding)?;
                        }
                    }
                }
            }
        }
    }

    Ok(())
}

fn save_workbook<'a>(view: &mut web_view::WebView<'a, StateData>) -> web_view::WVResult {
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

        for (_, finding) in &view.user_data().findings {
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

fn add_finding<'a>(view: &mut web_view::WebView<'a, StateData>, finding: &Finding) -> web_view::WVResult {
    //
    // Create a new finding table
    //

    let mut new_table = HtmlElement::new("table", "new_table");
    new_table.set_attribute("id", format!("finding{}", finding.id).as_str());

    let new_row = new_table.insert_row(0, "new_row");
    let new_cell = new_row.insert_cell(0, "new_cell");

    //
    // Create the title table for the new finding
    //

    let mut title_table = HtmlElement::new("table", "title_table");
    title_table.set_attribute("style", "table-layout: auto");

    let title_row = title_table.insert_row(0, "title_row");

    // ---------------------------------------------------

    let title_close_cell = title_row.insert_cell(0, "title_close_cell");

    let mut title_close_button = HtmlElement::new("button", "title_close_button");
    title_close_button.set_attribute("onclick", format!("external.invoke(\"remove_finding {}\")", finding.id).as_str());
    title_close_button.set_inner_html("X");

    title_close_cell.append_child(title_close_button);

    // ---------------------------------------------------

    let title_text_cell = title_row.insert_cell(1, "title_text_cell");

    let mut title_text_input = HtmlElement::new("input", "title_text_input");
    title_text_input.set_attribute("type", "text");
    title_text_input.set_attribute("id", format!("finding{}_title", finding.id).as_str());
    title_text_input.set_attribute("value", finding.title.as_str());
    title_text_input.set_attribute("style", "font-size: 150%");
    title_text_input.set_attribute("size", "80");
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

    let mut create_text_area = |name, text, string| {
        let mut header = HtmlElement::new("h4", format!("{}_header", name).as_str());
        header.set_inner_html(text);
        new_cell.append_child(header);
    
        let mut textarea = HtmlElement::new("textarea", format!("finding{}_{}_textarea", finding.id, name).as_str());
        textarea.set_inner_html(string);
        textarea.set_attribute("rows", "4");
        textarea.set_attribute("cols", "80");
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

    assert!(view.user_data_mut().findings.insert(finding.id, finding.clone()).is_none());

    findings.build(view)?;

    // Jump to the new finding
    view.eval(format!("window.location = '#finding{}'", finding.id).as_str()).unwrap();

    //
    // Create the table of contents link
    //

    let mut toc_findings = HtmlElement::get("toc_findings");
    
    let mut p = HtmlElement::new("p", "p");
    p.set_attribute("id", format!("finding{}_link_p", finding.id).as_str());

    let mut link = HtmlElement::new("a", "link");
    link.set_attribute("id", format!("finding{}_link", finding.id).as_str());
    link.set_attribute("href", format!("#finding{}", finding.id).as_str());
    link.set_inner_html(finding.title.as_str());

    p.append_child(link);

    toc_findings.append_child(p);
    toc_findings.build(view)
}

fn remove_finding<'a>(view: &mut web_view::WebView<'a, StateData>, id: usize) -> web_view::WVResult {
    if view.user_data_mut().findings.remove(&id).is_some() {
        let _ = view.user_data_mut().findings.remove(&id);

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

fn set_finding_title<'a>(view: &mut web_view::WebView<'a, StateData>, id: usize, title: &str) -> web_view::WVResult {
    if let Some(entry) = view.user_data_mut().findings.get_mut(&id) {
        entry.title = title.to_string();

        let mut finding_link = HtmlElement::get(format!("finding{}_link", id).as_str());
        finding_link.set_inner_html(title);
        finding_link.build(view)?;

        let mut finding_title = HtmlElement::get(format!("finding{}_title", id).as_str());
        finding_title.set_inner_html(title);
        finding_title.build(view)
    } else {
        Err(web_view::Error::Custom(Box::new(format!("No finding for id {} was found!", id))))
    }
}

fn set_finding_type<'a>(view: &mut web_view::WebView<'a, StateData>, id: usize, class: &str) -> web_view::WVResult {
    if let Some(entry) = view.user_data_mut().findings.get_mut(&id) {
        entry.class = class.to_string();

        let mut finding_type = HtmlElement::get(format!("finding{}_type", id).as_str());
        finding_type.set_inner_html(class);
        finding_type.build(view)
    } else {
        Err(web_view::Error::Custom(Box::new(format!("No finding for id {} was found!", id))))
    }
}

fn set_finding_severity<'a>(view: &mut web_view::WebView<'a, StateData>, id: usize, severity: &str) -> web_view::WVResult {
    if let Some(entry) = view.user_data_mut().findings.get_mut(&id) {
        entry.severity = match severity {
            "critical" => Some(report::Severity::Critical),
            "major" => Some(report::Severity::Major),
            "minor" => Some(report::Severity::Minor),
            "informational" => Some(report::Severity::Informational),
            _ => None
        };

        let mut option = HtmlElement::get(format!("finding{}_severity_{}_option", id, severity).as_str());
        option.set_selected(true);
        option.build(view)
    } else {
        Err(web_view::Error::Custom(Box::new(format!("No finding for id {} was found!", id))))
    }
}

fn set_finding_location<'a>(view: &mut web_view::WebView<'a, StateData>, id: usize, location: &str) -> web_view::WVResult {
    if let Some(entry) = view.user_data_mut().findings.get_mut(&id) {
        entry.location = location.to_string();

        let mut finding_location = HtmlElement::get(format!("finding{}_location", id).as_str());
        finding_location.set_inner_html(location);
        finding_location.build(view)
    } else {
        Err(web_view::Error::Custom(Box::new(format!("No finding for id {} was found!", id))))
    }
}

fn set_finding_description<'a>(view: &mut web_view::WebView<'a, StateData>, id: usize, description: &str) -> web_view::WVResult {
    if let Some(entry) = view.user_data_mut().findings.get_mut(&id) {
        entry.description = description.to_string();

        let mut finding_description = HtmlElement::get(format!("finding{}_description", id).as_str());
        finding_description.set_inner_html(description);
        finding_description.build(view)
    } else {
        Err(web_view::Error::Custom(Box::new(format!("No finding for id {} was found!", id))))
    }
}

fn set_finding_recommendation<'a>(view: &mut web_view::WebView<'a, StateData>, id: usize, recommendation: &str) -> web_view::WVResult {
    if let Some(entry) = view.user_data_mut().findings.get_mut(&id) {
        entry.recommendation = recommendation.to_string();

        let mut finding_recommendation = HtmlElement::get(format!("finding{}_recommendation", id).as_str());
        finding_recommendation.set_inner_html(recommendation);
        finding_recommendation.build(view)
    } else {
        Err(web_view::Error::Custom(Box::new(format!("No finding for id {} was found!", id))))
    }
}

fn set_finding_alleviation<'a>(view: &mut web_view::WebView<'a, StateData>, id: usize, alleviation: &str) -> web_view::WVResult {
    if let Some(entry) = view.user_data_mut().findings.get_mut(&id) {
        entry.alleviation = alleviation.to_string();

        let mut finding_alleviation = HtmlElement::get(format!("finding{}_alleviation", id).as_str());
        finding_alleviation.set_inner_html(alleviation);
        finding_alleviation.build(view)
    } else {
        Err(web_view::Error::Custom(Box::new(format!("No finding for id {} was found!", id))))
    }
}
