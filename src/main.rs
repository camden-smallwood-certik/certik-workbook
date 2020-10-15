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
            if arg.contains(' ') {
                // Tokenize the input command and parameters
                let mut iter = arg.split_whitespace();

                // Handle the input command
                match iter.next().unwrap() {
                    "remove_finding" => remove_finding(view, iter.next().unwrap().parse().unwrap())?,
                    "set_finding_title" => set_finding_title(view, iter.next().unwrap().parse().unwrap(), iter.next().unwrap())?,
                    _ => unimplemented!("{}", arg)
                }

                // Verify all parameters were used
                assert!(iter.next().is_none());
            } else {
                // Handle the input command
                match arg {
                    "create_finding" => create_finding(view)?,
                    "clear_findings" => clear_findings(view)?,
                    "load_workbook" => load_workbook(view)?,
                    "save_workbook" => save_workbook(view)?,
                    _ => unimplemented!("{}", arg)
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
        locations: vec![],
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

    title_text_input.set_attribute("onchange", format!("set_finding_title({})", finding.id).as_str());
    
    title_text_cell.append_child(title_text_input);

    // ---------------------------------------------------

    new_cell.append_child(title_table);

    //
    // Create the header table for the new finding
    //

    let mut header_table = HtmlElement::new("table", "header_table");

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

    header_type_cell.append_child(header_type_input);

    // ---------------------------------------------------

    let header_severity_cell = header_row2.insert_cell(1, "header_severity_cell");

    let mut header_severity_select = HtmlElement::new("select", "header_severity_select");

    let mut create_severity_option = |name, text| {
        let mut option = HtmlElement::new("option", format!("{}_option", name).as_str());
        option.set_attribute("value", name);
        option.set_inner_html(text);
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
    header_location_input.set_attribute("value", "");

    header_location_cell.append_child(header_location_input);

    //
    // Done building the header table for the new finding
    //

    new_cell.append_child(header_table);

    //
    // Create the text areas for the new finding
    //

    let mut create_text_area = |name, text| {
        let mut header = HtmlElement::new("h4", format!("{}_header", name).as_str());
        header.set_inner_html(text);
        new_cell.append_child(header);
    
        let mut textarea = HtmlElement::new("textarea", format!("{}_textarea", name).as_str());
        textarea.set_attribute("rows", "4");
        textarea.set_attribute("cols", "80");
        textarea.set_inner_html(finding.description.as_str());
        new_cell.append_child(textarea);
    };

    create_text_area("description", "Description:");
    create_text_area("recommendation", "Recommendation:");
    create_text_area("alleviation", "Alleviation:");

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

    let mut toc = HtmlElement::get("toc");
    
    let mut p = HtmlElement::new("p", "p");
    p.set_attribute("id", format!("finding{}_link", finding.id).as_str());

    let mut link = HtmlElement::new("a", "link");
    link.set_attribute("href", format!("#finding{}", finding.id).as_str());
    link.set_inner_html(finding.title.as_str());

    p.append_child(link);

    toc.append_child(p);
    toc.build(view)
}

fn remove_finding<'a>(view: &mut web_view::WebView<'a, StateData>, id: usize) -> web_view::WVResult {
    if view.user_data_mut().findings.remove(&id).is_some() {
        let _ = view.user_data_mut().findings.remove(&id);

        let mut finding = HtmlElement::get(format!("finding{}", id).as_str());
        finding.remove();
        finding.build(view)?;

        let mut link = HtmlElement::get(format!("finding{}_link", id).as_str());
        link.remove();
        link.build(view)
    } else {
        Err(web_view::Error::Custom(Box::new(format!("No finding for id {} was found!", id))))
    }
}

fn set_finding_title<'a>(view: &mut web_view::WebView<'a, StateData>, id: usize, title: &str) -> web_view::WVResult {
    if let Some(entry) = view.user_data_mut().findings.get_mut(&id) {
        entry.title = title.to_string();

        let mut link = HtmlElement::get(format!("finding{}_link", id).as_str());
        link.set_inner_html(title);
        link.build(view)
    } else {
        Err(web_view::Error::Custom(Box::new(format!("No finding for id {} was found!", id))))
    }
}
