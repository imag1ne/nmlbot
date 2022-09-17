use chrono::NaiveDate;
use serde_json::{json, Value};

pub fn title_database_property_object(content: &str) -> Value {
    json!({
        "type": "title",
        "title": [
            {
                "type": "text",
                "text": {
                    "content": content
                }
            }
        ]
    })
}

pub fn text_database_property_object(content: &str) -> Value {
    json!({
        "rich_text": [
            {
                "type": "text",
                "text": {
                    "content": content
                }
            }
        ]
    })
}

pub fn url_database_property_object(url: &str) -> Value {
    json!({ "url": url })
}

pub fn f64_number_database_property_object(number: f64) -> Value {
    json!({ "number": number })
}

pub fn u32_number_database_property_object(number: u32) -> Value {
    json!({ "number": number })
}

pub fn select_database_property_object(content: &str) -> Value {
    json!({
        "type": "select",
        "select": {
            "name": content
        }
    })
}

pub fn multi_select_database_property_object(contents: &[String]) -> Value {
    let contents = contents
        .iter()
        .map(|content| json!({ "name": content }))
        .collect::<Vec<_>>();

    json!({
        "type": "multi_select",
        "multi_select": contents
    })
}

pub fn date_database_property_object(date: &NaiveDate) -> Value {
    json!({
      "date": {
        "start": date.to_string()
      }
    })
}

pub fn new_database_object() -> Value {
    json!({
        "properties": {}
    })
}

pub fn file_object(url: &str) -> Value {
    json!({
        "type": "external",
        "external": {
            "url": url
        }
    })
}

pub fn parent_object(db_id: &str) -> Value {
    json!({
        "database_id": db_id,
        "type": "database_id"
    })
}
