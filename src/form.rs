use scraper::{Html, Selector};
use serde::Serialize;
use url::Url;

#[derive(Serialize, Debug)]
pub struct Field {
    name: String,
    r#type: String,
    value: Option<String>,
}

#[derive(Serialize, Debug)]
pub struct Form {
    action: Option<String>,
    method: Option<String>,
    fields: Vec<Field>,
}

pub fn extract_forms(html: &str, base_url: Option<&Url>) -> Vec<Form> {
    let doc = Html::parse_document(html);
    let form_selector = Selector::parse("form").unwrap();
    let input_selector = Selector::parse("input, textarea, select").unwrap();

    doc.select(&form_selector)
        .map(|form_el| {
            let action_attr = form_el.value().attr("action").map(|a| {
                base_url
                    .and_then(|b| b.join(a).ok())
                    .map(|u| u.to_string())
                    .unwrap_or_else(|| a.to_string())
            });

            let method_attr = form_el
                .value()
                .attr("method")
                .map(|m| m.to_ascii_uppercase());

            let fields = form_el
                .select(&input_selector)
                .filter_map(|inp| {
                    let name = inp.value().attr("name")?.to_string();
                    let t = inp.value().name();
                    let field_type = inp.value().attr("type").unwrap_or(t).to_string();
                    let value = inp.value().attr("value").map(|v| v.to_string());
                    Some(Field {
                        name,
                        r#type: field_type,
                        value,
                    })
                })
                .collect();

            Form {
                action: action_attr,
                method: method_attr,
                fields,
            }
        })
        .collect()
}

pub enum FieldType {
    Other,
    Id,
    Password,
}

pub fn classify_field(field: &Field) -> FieldType {
    if field.r#type == "hidden" {
        return FieldType::Other;
    }

    let name = field.name.to_lowercase();

    if ["pass", "password", "pwd"].iter().any(|k| name.contains(k)) || field.r#type == "password" {
        return FieldType::Password;
    }

    if ["user", "userid", "username", "id", "login", "email", "mail"]
        .iter()
        .any(|k| name.contains(k))
    {
        return FieldType::Id;
    }

    FieldType::Other
}
