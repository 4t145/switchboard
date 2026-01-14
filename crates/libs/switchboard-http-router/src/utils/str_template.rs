use std::{convert::Infallible, str::FromStr, sync::Arc};

#[derive(Debug, Clone)]
/// # examples
///
/// - general: `api/{version}/users/{user_id=guest}`
/// - include brackets: `api/{{version}}/users/brackets-here-{{not-capture}}`
///
pub struct StrTemplate {
    pub template: Arc<str>,
    pub segments: Arc<[PathTemplateSegment]>,
}

impl StrTemplate {
    pub fn render(&self, values: &std::collections::HashMap<&str, &str>) -> String {
        let mut result = String::new();
        for segment in self.segments.iter() {
            match segment {
                PathTemplateSegment::Static(s) => {
                    result.push_str(s);
                }
                PathTemplateSegment::Capture { key, default } => {
                    if let Some(value) = values.get(key.as_ref()) {
                        result.push_str(value);
                    } else if let Some(def) = default {
                        result.push_str(def);
                    }
                }
            }
        }
        result
    }
    pub fn is_literal(&self) -> bool {
        self.segments
            .iter()
            .all(|seg| matches!(seg, PathTemplateSegment::Static(_)))
    }
}

#[derive(Debug, Clone)]
pub enum PathTemplateSegment {
    Static(Arc<str>),
    Capture {
        key: Arc<str>,
        default: Option<Arc<str>>,
    },
}

impl FromStr for StrTemplate {
    type Err = Infallible;

    fn from_str(template: &str) -> Result<Self, Self::Err> {
        let mut segments = Vec::new();
        let mut chars = template.chars().peekable();
        let mut current_static = String::new();

        while let Some(&ch) = chars.peek() {
            match ch {
                '{' => {
                    // Check for escaped {{
                    chars.next(); // consume '{'
                    if let Some(&next_ch) = chars.peek() {
                        if next_ch == '{' {
                            current_static.push('{');
                            chars.next(); // consume second '{'
                            continue;
                        }
                    }

                    // Flush current static segment
                    if !current_static.is_empty() {
                        segments.push(PathTemplateSegment::Static(Arc::from(
                            current_static.clone(),
                        )));
                        current_static.clear();
                    }

                    // Parse capture segment
                    let mut key = String::new();
                    let mut default = None;
                    while let Some(&c) = chars.peek() {
                        if c == '}' {
                            chars.next(); // consume '}'
                            break;
                        } else if c == '=' {
                            chars.next(); // consume '='
                            let mut default_value = String::new();
                            while let Some(&dc) = chars.peek() {
                                if dc == '}' {
                                    break;
                                }
                                default_value.push(dc);
                                chars.next();
                            }
                            default = Some(Arc::from(default_value));
                        } else {
                            key.push(c);
                            chars.next();
                        }
                    }
                    segments.push(PathTemplateSegment::Capture {
                        key: Arc::from(key),
                        default,
                    });
                }
                '}' => {
                    // Check for escaped }}
                    chars.next(); // consume '}'
                    if let Some(&next_ch) = chars.peek() {
                        if next_ch == '}' {
                            current_static.push('}');
                            chars.next(); // consume second '}'
                            continue;
                        }
                    }
                    // Unmatched '}', treat as literal
                    current_static.push('}');
                }
                _ => {
                    current_static.push(ch);
                    chars.next();
                }
            }
        }

        // Flush remaining static segment
        if !current_static.is_empty() {
            segments.push(PathTemplateSegment::Static(Arc::from(current_static)));
        }

        Ok(StrTemplate {
            template: Arc::from(template),
            segments: Arc::from(segments),
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_str_template_parsing() {
        let template_str = "api/{version}/users/{user_id=guest}/data/{{brackets}}";
        let template = StrTemplate::from_str(template_str).unwrap();

        assert_eq!(template.segments.len(), 5);

        match &template.segments[0] {
            PathTemplateSegment::Static(s) => assert_eq!(s.as_ref(), "api/"),
            _ => panic!("Expected static segment"),
        }

        match &template.segments[1] {
            PathTemplateSegment::Capture { key, default } => {
                assert_eq!(key.as_ref(), "version");
                assert!(default.is_none());
            }
            _ => panic!("Expected capture segment"),
        }

        match &template.segments[2] {
            PathTemplateSegment::Static(s) => assert_eq!(s.as_ref(), "/users/"),
            _ => panic!("Expected static segment"),
        }

        match &template.segments[3] {
            PathTemplateSegment::Capture { key, default } => {
                assert_eq!(key.as_ref(), "user_id");
                assert_eq!(default.as_ref().unwrap().as_ref(), "guest");
            }
            _ => panic!("Expected capture segment"),
        }

        match &template.segments[4] {
            PathTemplateSegment::Static(s) => assert_eq!(s.as_ref(), "/data/{brackets}"),
            _ => panic!("Expected static segment"),
        }
    }

    #[test]
    fn test_str_template_rendering() {
        let template_str = "api/{version}/users/user-{user_id=guest}/data/{{brackets}}";
        let template = StrTemplate::from_str(template_str).unwrap();

        let mut values = std::collections::HashMap::new();
        values.insert("version", "v1");
        values.insert("user_id", "alice");

        let rendered = template.render(&values);
        assert_eq!(rendered, "api/v1/users/user-alice/data/{brackets}");

        let mut values_missing_user = std::collections::HashMap::new();
        values_missing_user.insert("version", "v2");

        let rendered_missing_user = template.render(&values_missing_user);
        assert_eq!(
            rendered_missing_user,
            "api/v2/users/user-guest/data/{brackets}"
        );
    }
}
