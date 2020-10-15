use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq)]
pub struct HtmlElement {
    pub get: bool,
    pub remove: bool,
    pub class: String,
    pub name: String,
    pub inner: Option<String>,
    pub attrs: HashMap<String, String>,
    pub rows: Vec<HtmlElement>,
    pub cells: Vec<HtmlElement>,
    pub children: Vec<HtmlElement>,
}

impl HtmlElement {
    pub fn new(class: &str, name: &str) -> Self {
        Self {
            get: false,
            remove: false,
            class: class.to_string(),
            name: name.to_string(),
            inner: None,
            attrs: HashMap::new(),
            rows: vec![],
            cells: vec![],
            children: vec![],
        }
    }

    pub fn get(name: &str) -> Self {
        Self {
            get: true,
            remove: false,
            class: "".to_string(),
            name: name.to_string(),
            inner: None,
            attrs: HashMap::new(),
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
                        format!("document.getElementById('{}')", self.name)
                    } else {
                        format!("document.createElement('{}')", self.class)
                    }
                ).as_str()
            );
        }

        if self.remove {
            js.push_str(format!("{}.remove();", self.name).as_str());
            return;
        }

        if let Some(inner) = &self.inner {
            js.push_str(format!("{}.innerHTML = '{}';", self.name, inner).as_str());
        }

        for attr in &self.attrs {
            js.push_str(format!("{}.setAttribute('{}', '{}');", self.name, attr.0, attr.1).as_str());
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
