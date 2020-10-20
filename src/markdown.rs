use comrak::{
    nodes::{AstNode, NodeValue},
};

use crate::{
    StateData,
    report::{Finding, Severity},
};

use std::str;

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub enum MarkdownHeader {
    Description,
    Recommendation,
    Alleviation
}

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub struct MarkdownParser {
    pub findings: Vec<Finding>,
    pub current_finding: Option<Finding>,
    pub current_header: Option<MarkdownHeader>,
    pub current_header_level: Option<u32>,
    pub current_table_cell: Option<u32>
}

impl MarkdownParser {
    pub fn new() -> Self {
        Self {
            findings: vec![],
            current_finding: None,
            current_header: None,
            current_header_level: None,
            current_table_cell: None
        }
    }

    fn parse_severity(input: &str) -> Option<Severity> {
        match input.to_lowercase().as_str() {
            "critical" => Some(Severity::Critical),
            "major" => Some(Severity::Major),
            "minor" => Some(Severity::Minor),
            "informational" => Some(Severity::Informational),
            _ => None
        }
    }

    pub fn parse_ast_node<'a>(&mut self, state: &mut StateData, node: &'a AstNode<'a>) {
        match &mut node.data.borrow_mut().value {
            &mut NodeValue::HtmlBlock(ref mut block) => {
                match str::from_utf8(block.literal.as_slice()).unwrap_or("") {
                    s if s.starts_with("<section id=\"") => {
                        self.current_finding = Some(Finding {
                            id: { state.current_finding_id += 1; state.current_finding_id },
                            title: String::new(),
                            class: String::new(),
                            severity: Self::parse_severity(s.split('"').nth(1).unwrap()),
                            location: String::new(),
                            description: String::new(),
                            recommendation: String::new(),
                            alleviation: String::new()
                        });
                    },

                    s if s.starts_with("</section>") => {
                        self.findings.push(self.current_finding.clone().unwrap());
                        self.current_finding = None;
                        self.current_header = None;
                        self.current_header_level = None;
                    }

                    s => println!("Unknown html block: {}", s)
                }
            }

            &mut NodeValue::Heading(ref mut heading) => {
                assert!(self.current_finding.is_some());
                self.current_header_level = Some(heading.level);
            }

            &mut NodeValue::Code(ref mut code) => {
                assert!(self.current_finding.is_some());
                if let Some(3) = self.current_header_level {
                    self.current_finding.as_mut().unwrap().title.push_str(
                        format!(" `{}` ", str::from_utf8(code).unwrap()).as_str()
                    );
                } else {
                    if let Some(header) = self.current_header {
                        match header {
                            MarkdownHeader::Description => if let Some(ref mut finding) = self.current_finding {
                                finding.description.push_str(format!("`{}`", str::from_utf8(code).unwrap()).as_str());
                            }
                            
                            MarkdownHeader::Recommendation => if let Some(ref mut finding) = self.current_finding {
                                finding.recommendation.push_str(format!("`{}`", str::from_utf8(code).unwrap()).as_str());
                            }
                            
                            MarkdownHeader::Alleviation => if let Some(ref mut finding) = self.current_finding {
                                finding.alleviation.push_str(format!("`{}`", str::from_utf8(code).unwrap()).as_str());
                            }
                        }
                    }
                }
            }

            &mut NodeValue::CodeBlock(ref mut code) => {
                assert!(self.current_finding.is_some());
                if let Some(3) = self.current_header_level {
                    self.current_finding.as_mut().unwrap().title.push_str(
                        format!(" `{}` ", str::from_utf8(code.literal.as_slice()).unwrap()).as_str()
                    );
                } else {
                    if let Some(header) = self.current_header {
                        match header {
                            MarkdownHeader::Description => if let Some(ref mut finding) = self.current_finding {
                                finding.description.push_str(format!("\n```\n{}```\n", str::from_utf8(code.literal.as_slice()).unwrap()).as_str());
                            }
                            
                            MarkdownHeader::Recommendation => if let Some(ref mut finding) = self.current_finding {
                                finding.recommendation.push_str(format!("\n```\n{}```\n", str::from_utf8(code.literal.as_slice()).unwrap()).as_str());
                            }
                            
                            MarkdownHeader::Alleviation => if let Some(ref mut finding) = self.current_finding {
                                finding.alleviation.push_str(format!("\n```\n{}```\n", str::from_utf8(code.literal.as_slice()).unwrap()).as_str());
                            }
                        }
                    }
                }
            }

            &mut NodeValue::Table(_) => self.current_header_level = None,
            &mut NodeValue::TableRow(false) => self.current_table_cell = Some(0),

            &mut NodeValue::Text(ref mut text) => {
                assert!(self.current_finding.is_some());
                if let Some(cell) = self.current_table_cell {
                    match cell {
                        0 => if let Some(ref mut finding) = self.current_finding {
                            finding.class.push_str(str::from_utf8(text).unwrap());
                            self.current_table_cell = Some(1);
                        },
                        1 => if let Some(ref mut _finding) = self.current_finding {
                            self.current_table_cell = Some(2);
                        },
                        2 => if let Some(ref mut finding) = self.current_finding {
                            finding.location.push_str(str::from_utf8(text).unwrap());
                            self.current_table_cell = None; // TODO: Check if alleviation exists?
                        },
                        3 => {
                            // TODO: Alleviation
                        },
                        _ => println!("Unhandled finding header table cell: {}", cell)
                    }
                } else {
                    if let Some(level) = self.current_header_level {
                        match level {
                            3 => if let Some(ref mut finding) = self.current_finding {
                                finding.title.push_str(str::from_utf8(text).unwrap());
                            }

                            4 => match str::from_utf8(text).unwrap() {
                                "Description:" => {
                                    self.current_header = Some(MarkdownHeader::Description);
                                    self.current_header_level = None;
                                }

                                "Recommendation:" => {
                                    self.current_header = Some(MarkdownHeader::Recommendation);
                                    self.current_header_level = None;
                                }

                                "Alleviation:" => {
                                    self.current_header = Some(MarkdownHeader::Alleviation);
                                    self.current_header_level = None;
                                }

                                _ => ()
                            }

                            _ => ()
                        }
                    } else {
                        if let Some(header) = self.current_header {
                            match header {
                                MarkdownHeader::Description => if let Some(ref mut finding) = self.current_finding {
                                    finding.description.push_str(str::from_utf8(text).unwrap());
                                }
                                
                                MarkdownHeader::Recommendation => if let Some(ref mut finding) = self.current_finding {
                                    finding.recommendation.push_str(str::from_utf8(text).unwrap());
                                }
                                
                                MarkdownHeader::Alleviation => if let Some(ref mut finding) = self.current_finding {
                                    finding.alleviation.push_str(str::from_utf8(text).unwrap());
                                }
                            }
                        } else {
                            println!("Unused text: {}", str::from_utf8(text).unwrap());
                        }
                    }
                }
            }
            node => println!("Unknown node: {:?}", node),
        }

        for c in node.children() {
            self.parse_ast_node(state, c);
        }

        self.findings.sort_by(|lhs, rhs| lhs.id.cmp(&rhs.id));
    }
}
