use crate::mongo::Website;

pub fn format_website_list(websites: &[Website]) -> String {
    if websites.is_empty() {
        return "You aren't tracking any sites yet.".to_string();
    }

    let mut table = "Here are your tracked domains:\n\n<pre>".to_string();
    table.push_str(&format!("{:<40} | {:<6}\n", "URL", "Status"));
    table.push_str(&"-".repeat(50));
    table.push('\n');

    for site in websites {
        table.push_str(&format!("{:<40} | {:<6}\n", site.url, site.status));
    }
    table.push_str("</pre>");

    table
}
