#[derive(Copy, Clone, Debug, PartialEq, PartialOrd, Deserialize, Serialize)]
pub enum Severity {
    Minor,
    Major,
    Critical,
    Informational
}

#[derive(Clone, Debug, PartialEq, PartialOrd, Deserialize, Serialize)]
pub struct Location {
    pub file: String,
    pub lines: Vec<usize>
}

#[derive(Clone, Debug, PartialEq, PartialOrd, Deserialize, Serialize)]
pub struct Auditor {
    pub name: String,
    pub email: String
}

#[derive(Clone, Debug, PartialEq, PartialOrd, Deserialize, Serialize)]
pub struct Finding {
    pub id: usize,
    pub title: String,
    pub class: String,
    pub severity: Option<Severity>,
    pub locations: Vec<Location>,
    pub description: String,
    pub recommendation: String,
    pub alleviation: String
}

#[derive(Clone, Debug, PartialEq, PartialOrd, Deserialize, Serialize)]
pub struct Report {
    pub title: String,
    pub auditors: Vec<Auditor>,
    pub start_time: String,
    pub delivery_time: String,
    pub repository: String,
    pub commit_hashes: Vec<String>,
    pub overview: String,
    pub findings: Vec<Finding>
}
