use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq)]
pub struct HtmlElement {
    pub get: bool,
    pub remove: bool,
    pub disabled: Option<bool>,
    pub class: String,
    pub name: String,
    pub inner: Option<String>,
    pub text: Option<String>,
    pub value: Option<String>,
    pub selected: Option<bool>,
    pub selected_index: Option<usize>,
    pub attrs: HashMap<String, String>,
    pub fields: HashMap<String, String>,
    pub rows: Vec<HtmlElement>,
    pub cells: Vec<HtmlElement>,
    pub children: Vec<HtmlElement>,
}

impl HtmlElement {
    pub fn new(class: &str, name: &str) -> Self {
        Self {
            get: false,
            remove: false,
            disabled: None,
            class: class.to_string(),
            name: name.to_string(),
            inner: None,
            text: None,
            value: None,
            selected: None,
            selected_index: None,
            attrs: HashMap::new(),
            fields: HashMap::new(),
            rows: vec![],
            cells: vec![],
            children: vec![],
        }
    }

    pub fn get(name: &str) -> Self {
        Self {
            get: true,
            remove: false,
            disabled: None,
            class: "".to_string(),
            name: name.to_string(),
            inner: None,
            text: None,
            value: None,
            selected: None,
            selected_index: None,
            attrs: HashMap::new(),
            fields: HashMap::new(),
            rows: vec![],
            cells: vec![],
            children: vec![],
        }
    }

    pub fn remove(&mut self) {
        self.remove = true;
    }

    pub fn set_attribute(&mut self, name: &str, value: &str) {
        let _ = self.attrs.insert(name.to_string(), value.to_string());
    }

    pub fn set_field(&mut self, name: &str, value: &str) {
        let _ = self.fields.insert(name.to_string(), value.to_string());
    }

    pub fn insert_row(&mut self, index: usize, name: &str) -> &mut HtmlElement {
        self.rows.insert(index, HtmlElement::new("tr", name));
        &mut self.rows[index]
    }

    pub fn insert_cell(&mut self, index: usize, name: &str) -> &mut HtmlElement {
        self.cells.insert(index, HtmlElement::new("td", name));
        &mut self.cells[index]
    }

    pub fn set_inner_html(&mut self, html: &str) {
        self.inner = Some(html.to_string());
    }

    pub fn set_text(&mut self, text: &str) {
        self.text = Some(text.to_string());
    }

    pub fn set_value(&mut self, value: &str) {
        self.value = Some(value.to_string());
    }

    pub fn set_disabled(&mut self, disabled: bool) {
        self.disabled = Some(disabled);
    }

    pub fn set_selected(&mut self, selected: bool) {
        self.selected = Some(selected);
    }

    pub fn set_selected_index(&mut self, index: Option<usize>) {
        self.selected_index = index;
    }

    pub fn append_child(&mut self, child: HtmlElement) {
        self.children.push(child);
    }

    pub fn build<'a, T: 'a>(&self, view: &mut web_view::WebView<'a, T>) -> web_view::WVResult {
        let mut js = String::new();
        self.build_js(&mut js);
        view.eval(js.as_str())
    }

    fn build_js(&self, js: &mut String) {
        if self.class != "tr" && self.class != "td" {
            js.push_str(
                format!("var {} = {};", self.name,
                    if self.get {
                        format!("document.getElementById('{}')", self.name.replace("'", "\\'").replace("\r\n", "\n").replace("\n", "\\n"))
                    } else {
                        format!("document.createElement('{}')", self.class.replace("'", "\\'").replace("\r\n", "\n").replace("\n", "\\n"))
                    }
                ).as_str()
            );
        }

        if self.remove {
            js.push_str(format!("{}.remove();", self.name).as_str());
            return;
        }

        if let Some(ref inner) = self.inner {
            js.push_str(format!("{}.innerHTML = '{}';", self.name, inner.replace("'", "\\'").replace("\r\n", "\n").replace("\n", "\\n")).as_str());
        }

        if let Some(ref text) = self.text {
            js.push_str(format!("{}.text = '{}';", self.name, text.replace("'", "\\'").replace("\r\n", "\n").replace("\n", "\\n")).as_str());
        }

        if let Some(ref value) = self.value {
            js.push_str(format!("{}.value = '{}';", self.name, value.replace("'", "\\'").replace("\r\n", "\n").replace("\n", "\\n")).as_str());
        }

        if let Some(true) = self.selected {
            js.push_str(format!("{}.selected = true;", self.name).as_str());
        }

        match self.disabled {
            Some(true) => js.push_str(format!("{}.disabled = true;", self.name).as_str()),
            Some(false) => js.push_str(format!("{}.disabled = false;", self.name).as_str()),
            None => ()
        }

        for attr in &self.attrs {
            js.push_str(format!("{}.setAttribute('{}', '{}');", self.name, attr.0.replace("'", "\\'").replace("\r\n", "\n").replace("\n", "\\n"), attr.1.replace("'", "\\'").replace("\r\n", "\n").replace("\n", "\\n")).as_str());
        }

        for field in &self.fields {
            js.push_str(format!("{}.{} = '{}';", self.name, field.0, field.1.replace("'", "\\'").replace("\r\n", "\n").replace("\n", "\\n")).as_str());
        }

        for (index, row) in self.rows.iter().enumerate() {
            js.push_str(format!("var {} = {}.insertRow({});", row.name, self.name, index).as_str());
            row.build_js(js);
        }

        for (index, cell) in self.cells.iter().enumerate() {
            js.push_str(format!("var {} = {}.insertCell({});", cell.name, self.name, index).as_str());
            cell.build_js(js);
        }

        for child in &self.children {
            child.build_js(js);
            js.push_str(format!("{}.appendChild({});", self.name, child.name).as_str());
        }
    }
}
