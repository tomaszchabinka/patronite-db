use std::{collections::HashSet, vec};

use regex::Regex;
use reqwest::blocking;
use scraper::{Html, Selector};

use std::str::FromStr;

const PATRONITE_URL: &str = "https://patronite.pl";

pub struct Category {
    pub id: u8,
    pub name: String,
    pub url: String,
}

pub fn get_list_of_categories() -> Vec<Category> {
    let document = get_html_document("/kategoria/47/polityka", 1);
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

            let id = url.split('/').nth(2).unwrap().parse::<u8>().unwrap();

            let category = Category {
                id,
                name: name.trim().to_owned(),
                url,
            };

            category
        })
        .collect::<Vec<Category>>();
}

#[derive(Debug)]
pub struct CreatorSummary {
    pub name: String,
    pub tags: Vec<String>,
    pub category_id: u8,
    pub number_of_patrons: u16,
    pub monthly_revenue: u32,
    pub total_revenue: u32,
    pub is_recommended: bool,
    pub url: String,
    pub image_url: String,
    pub timestamp: u32,
}

pub fn get_list_of_creators(category: &Category) -> Vec<CreatorSummary> {
    println!("Fetching creators for category: {}", category.name);

    let mut page: u16 = 1;
    let mut our_choice_selection = vec![];
    let mut all_creators = vec![];

    loop {
        println!("Fetching page {}... ", page);
        let document: Html = get_html_document(&category.url, page);

        if page == 1 {
            let our_selection_header =
                Selector::parse("div.author__list--header").expect("Failed to parse CSS selector");

            if let Some(_our_selection_header) = document.select(&our_selection_header).next() {
                let _our_choice_selector = Selector::parse(
            ".less-horizontal-padding > div.row:nth-child(2) > div.col-xs-12 > div.author__list > div.carousel-cell > a",
                ).expect("Failed to parse CSS selector");

                our_choice_selection.append(
                    &mut document
                        .select(&_our_choice_selector)
                        .map(|element| {
                            get_creator_summary(&element, category.id, &HashSet::<&String>::new())
                        })
                        .collect::<Vec<CreatorSummary>>(),
                );
            }
        }

        let child_id = if page == 1 { 3 } else { 2 };
        let selector = format!(".less-horizontal-padding > div.row:nth-child({child_id}) > div.col-xs-12 > div.author__list > div.carousel-cell > a");
        let all_selector = Selector::parse(&selector).expect("Failed to parse CSS selector");

        let favourite_ids = our_choice_selection
            .iter()
            .map(|creator| &creator.url)
            .collect::<HashSet<&String>>();

        all_creators.append(
            &mut document
                .select(&all_selector)
                .map(|element| get_creator_summary(&element, category.id, &favourite_ids))
                .collect::<Vec<CreatorSummary>>(),
        );

        let next_page_selector =
            Selector::parse("ul.pagination > li.pagination__item > a").unwrap();

        let next_page_button = document
            .select(&next_page_selector)
            .map(|element| element.text().collect::<String>().trim().to_owned())
            .filter(|text| text == "Następne »")
            .collect::<Vec<String>>();

        if next_page_button.is_empty() {
            break;
        }
        page += 1;
    }

    all_creators
}

fn get_html_document(url: &str, page: u16) -> Html {
    let response =
        blocking::get(format!("{PATRONITE_URL}{url}?page={page}")).expect("Failed to send request");

    // Read the response body as a string
    let body = response.text().expect("Failed to read response body");
    Html::parse_document(&body)
}

fn get_creator_summary(
    element: &scraper::ElementRef,
    category_id: u8,
    favourite_ids: &HashSet<&String>,
) -> CreatorSummary {
    let numbers = element
        .select(&Selector::parse("div.card__content--numbers > div > span:nth-child(1)").unwrap())
        .map(|tag| tag.text().collect::<String>().trim().to_owned())
        .collect::<Vec<String>>();

    let url = element.attr("href").unwrap().to_owned();

    CreatorSummary {
        name: element
            .select(&Selector::parse("div.card__content--name").unwrap())
            .next()
            .unwrap()
            .text()
            .collect::<String>()
            .trim()
            .to_owned(),
        tags: element
            .select(&Selector::parse("div.card__content--tags > span").unwrap())
            .map(|tag| tag.text().collect::<String>().trim().to_owned())
            .collect::<Vec<String>>(),
        category_id,
        number_of_patrons: if !numbers.is_empty() {
            parse_number(&numbers[0]) as u16
        } else {
            0
        },
        monthly_revenue: if numbers.len() > 1 {
            parse_number(&numbers[1])
        } else {
            0
        },
        total_revenue: if numbers.len() > 2 {
            parse_number(&numbers[2])
        } else {
            0
        },
        is_recommended: favourite_ids.contains(&url),
        url,
        image_url: element
            .select(&Selector::parse("div.author__card--image > span > img").unwrap())
            .next()
            .unwrap()
            .attr("data-src")
            .unwrap()
            .to_owned(),
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as u32,
    }
}

fn parse_number(s: &str) -> u32 {
    let re = Regex::new(r"^(\d+(\.\d+)?)\s*(tys\.|mln)?\s*(zł)?$").unwrap();
    let captures = re.captures(s);

    if captures.is_none() {
        println!("Failed to parse number: {}", s);
        return 0;
    }

    let captures = captures.unwrap();

    let number = f64::from_str(&captures[1]).unwrap();
    let multiplier = match captures.get(3).map(|m| m.as_str()) {
        Some("tys.") => 1_000,
        Some("mln") => 1_000_000,
        _ => 1,
    };
    (number * multiplier as f64) as u32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_number() {
        assert_eq!(parse_number("7112"), 7112);
        assert_eq!(parse_number("112430 zł"), 112430);
        assert_eq!(parse_number("4.23 mln zł"), 4230000);
        assert_eq!(parse_number("40 mln zł"), 40000000);
        assert_eq!(parse_number("197 tys. zł"), 197000);
    }
}
