pub mod html;
use crate::html::HtmlElement;
use std::time::SystemTime;

pub struct Report {
    pub title: String,
    pub auditors: Vec<Auditor>,
    pub start_time: SystemTime,
    pub delivery_time: SystemTime,
    pub repository: String,
    pub commit_hashes: Vec<String>,
    pub overview: String,
    pub findings: Vec<Finding>
}

pub struct Auditor {
    pub name: String,
    pub email: String
}

pub struct Finding {
    pub title: String,
    pub class: String,
    pub severity: Severity,
    pub locations: Vec<Location>,
    pub description: String,
    pub recommendation: String,
    pub alleviation: String
}

pub enum Severity {
    Minor,
    Major,
    Critical,
    Informational
}

pub struct Location {
    pub file: String,
    pub lines: Vec<usize>
}

#[derive(Debug)]
pub struct StateData {
    pub current_finding_id: usize
}

fn main() {
    /*match tinyfiledialogs::open_file_dialog("Select a JSON workbook", "", None) {
        Some(file) => println!("{}", file),
        None => ()
    };*/

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
        .user_data(StateData {
            current_finding_id: 0
        })
        .invoke_handler(|view, arg| {
            if arg.contains(' ') {
                // Tokenize the input command and parameters
                let mut iter = arg.split_whitespace();

                // Handle the input command
                match iter.next().unwrap() {
                    "remove_finding" => removing_finding(view, iter.next().unwrap().parse().unwrap())?,
                    "set_finding_title" => set_finding_title(view, iter.next().unwrap().parse().unwrap(), iter.next().unwrap())?,
                    _ => unimplemented!("{}", arg)
                }

                // Verify all parameters were used
                assert!(iter.next().is_none());
            } else {
                // Handle the input command
                match arg {
                    "create_finding" => create_finding(view)?,
                    "exit" => view.exit(),
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

    //
    // TODO: Register finding identifier
    //

    //
    // Create a new finding table
    //

    let mut new_table = HtmlElement::new("table", "new_table");
    new_table.set_attribute("id", format!("finding{}", state.current_finding_id).as_str());

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
    title_close_button.set_attribute("onclick", format!("external.invoke(\"remove_finding {}\")", state.current_finding_id).as_str());
    title_close_button.set_inner_html("X");

    title_close_cell.append_child(title_close_button);

    // ---------------------------------------------------

    let title_text_cell = title_row.insert_cell(1, "title_text_cell");

    let mut title_text_input = HtmlElement::new("input", "title_text_input");
    title_text_input.set_attribute("type", "text");
    title_text_input.set_attribute("id", format!("finding{}_title", state.current_finding_id).as_str());
    title_text_input.set_attribute("value", format!("Title{}", state.current_finding_id).as_str());
    title_text_input.set_attribute("style", "font-size: 150%");
    title_text_input.set_attribute("size", "80");

    title_text_input.set_attribute("onchange", format!("set_finding_title({})", state.current_finding_id).as_str());
    
    title_text_cell.append_child(title_text_input);

    // ---------------------------------------------------

    new_cell.append_child(title_table);

    //
    // Create the header table for the new finding
    //

    let mut header_table = HtmlElement::new("table", "header_table");

    // ---------------------------------------------------

    let header_row1 = header_table.insert_row(0, "header_row1");

    let header_cell1 = header_row1.insert_cell(0, "header_cell1");
    header_cell1.set_attribute("style", "padding-top: 6.5px; padding-bottom: 6.5px");
    header_cell1.set_inner_html("Type");

    let header_cell2 = header_row1.insert_cell(1, "header_cell2");
    header_cell2.set_attribute("style", "padding-top: 6.5px; padding-bottom: 6.5px");
    header_cell2.set_inner_html("Severity");

    let header_cell3 = header_row1.insert_cell(2, "header_cell3");
    header_cell3.set_attribute("style", "padding-top: 6.5px; padding-bottom: 6.5px");
    header_cell3.set_inner_html("Location");

    // ---------------------------------------------------

    let header_row2 = header_table.insert_row(1, "header_row2");

    let header_type_cell = header_row2.insert_cell(0, "header_type_cell");

    let mut header_type_input = HtmlElement::new("input", "header_type_input");
    header_type_input.set_attribute("type", "text");
    header_type_input.set_attribute("value", "Type");

    header_type_cell.append_child(header_type_input);

    // ---------------------------------------------------

    let header_severity_cell = header_row2.insert_cell(1, "header_severity_cell");

    let mut header_severity_select = HtmlElement::new("select", "header_severity_select");

    let mut critical_option = HtmlElement::new("option", "critical_option");
    critical_option.set_attribute("value", "critical");
    critical_option.set_inner_html("Critical");
    header_severity_select.append_child(critical_option);

    let mut major_option = HtmlElement::new("option", "major_option");
    major_option.set_attribute("value", "major");
    major_option.set_inner_html("Major");
    header_severity_select.append_child(major_option);

    let mut minor_option = HtmlElement::new("option", "minor_option");
    minor_option.set_attribute("value", "minor");
    minor_option.set_inner_html("Minor");
    header_severity_select.append_child(minor_option);

    let mut informational_option = HtmlElement::new("option", "informational_option");
    informational_option.set_attribute("value", "informational");
    informational_option.set_inner_html("Informational");
    header_severity_select.append_child(informational_option);

    header_severity_cell.append_child(header_severity_select);
    
    // ---------------------------------------------------

    let header_location_cell = header_row2.insert_cell(2, "header_location_cell");

    let mut header_location_input = HtmlElement::new("input", "header_location_input");
    header_location_input.set_attribute("type", "text");
    header_location_input.set_attribute("value", "Location");

    header_location_cell.append_child(header_location_input);

    //
    // Done building the header table for the new finding
    //

    new_cell.append_child(header_table);

    //
    // Create the "description" text area for the new finding
    //

    let mut desc_head1 = HtmlElement::new("h4", "desc_head1");
    desc_head1.set_inner_html("Description:");
    new_cell.append_child(desc_head1);

    let mut desc_text1 = HtmlElement::new("textarea", "desc_text1");
    desc_text1.set_attribute("rows", "4");
    desc_text1.set_attribute("cols", "80");
    new_cell.append_child(desc_text1);

    //
    // Create the "recommendation" text area for the new finding
    //

    let mut desc_head2 = HtmlElement::new("h4", "desc_head2");
    desc_head2.set_inner_html("Recommendation:");
    new_cell.append_child(desc_head2);

    let mut desc_text2 = HtmlElement::new("textarea", "desc_text2");
    desc_text2.set_attribute("rows", "4");
    desc_text2.set_attribute("cols", "80");
    new_cell.append_child(desc_text2);

    //
    // Create the "alleviation" text area for the new finding
    //

    let mut desc_head3 = HtmlElement::new("h4", "desc_head3");
    desc_head3.set_inner_html("Alleviation:");
    new_cell.append_child(desc_head3);

    let mut desc_text3 = HtmlElement::new("textarea", "desc_text3");
    desc_text3.set_attribute("rows", "4");
    desc_text3.set_attribute("cols", "80");
    new_cell.append_child(desc_text3);

    //
    // Done building new finding table
    //

    let mut findings = HtmlElement::get("findings");
    findings.append_child(new_table);
    findings.build(view)
}

fn removing_finding<'a>(view: &mut web_view::WebView<'a, StateData>, index: usize) -> web_view::WVResult {
    //
    // TODO: Unregister finding identifier
    //

    let mut finding = HtmlElement::get(format!("finding{}", index).as_str());
    finding.remove();
    finding.build(view)
}

fn set_finding_title<'a>(_view: &mut web_view::WebView<'a, StateData>, index: usize, title: &str) -> web_view::WVResult {
    println!("Setting finding {}'s title to \"{}\"", index, title);
    Ok(())
}
