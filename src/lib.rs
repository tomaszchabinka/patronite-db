use reqwest::blocking;
use scraper::{Html, Selector};

const PATRONITE_URL: &str = "https://patronite.pl";

pub struct Category {
    pub id: u32,
    pub name: String,
    pub url: String,
}

pub fn get_list_of_categories() -> Vec<Category> {
    let response = blocking::get(String::from(PATRONITE_URL) + "/kategoria/47/polityka")
        .expect("Failed to send request");

    // Read the response body as a string
    let body = response.text().expect("Failed to read response body");

    // Parse the HTML body
    let document = Html::parse_document(&body);

    let selector = Selector::parse("div.tags > div").expect("Failed to parse CSS selector");

    return document
        .select(&selector)
        .map(|element| {
            let url = element
                .select(&scraper::Selector::parse("a").unwrap())
                .next()
                .and_then(|a| a.value().attr("href"))
                .map(str::to_owned);

            let name = element.text().collect::<String>();

            let url: String = url.unwrap().replace(PATRONITE_URL, "");

            let id = url.split('/').nth(2).unwrap().parse::<u32>().unwrap();

            let category = Category {
                id,
                name: name.trim().to_owned(),
                url,
            };

            category
        })
        .collect::<Vec<Category>>();
}
