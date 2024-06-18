pub mod goodreads_api {
    use std::io;

    // use reqwest::get;
    use reqwest;

    use scraper::{Html, Selector};

    #[derive(Debug, PartialEq)]
    pub struct GoodreadsBook {
        title: String,
        authors: Vec<String>,
        pages: u64,
        series: Option<String>,
        url: String
    }

    impl GoodreadsBook {

        pub fn new(title: String, authors: Vec<String>, pages: u64, series: Option<String>, url: String) -> Self {
            Self {title, authors, pages, series, url}
        }

        pub fn search(query: &str) -> Vec<Self> {

            let url = format!("https://www.goodreads.com/search?q={}&tab=books", query);
            
            let response = reqwest::blocking::get(&url).unwrap().text().unwrap();

            let document = Html::parse_document(&response);

            let tr_selector = scraper::Selector::parse(r#"tr[itemscope][itemtype="http://schema.org/Book"]"#).unwrap();

            let a_selector = scraper::Selector::parse("a:not([class])").unwrap();

            for tr_element in document.select(&tr_selector) {
                
                for a_element in tr_element.select(&a_selector) {
                    if let Some(title) = a_element.value().attr("title") {
                        println!("{}", title);
                    }
                }
            }
            vec![]
        }
    }

       

}

mod tests {
    use goodreads_api::GoodreadsBook;

    use super::*;

    #[test]
    fn test_search() {
        let books = goodreads_api::GoodreadsBook::search("The Hobbit");
        assert_eq!(books, vec![GoodreadsBook::new("The Hobbit".to_string(), vec!["J.R.R. Tolkien".to_string()], 310, Option::None, "https://www.goodreads.com/book/show/5907.The_Hobbit?from_search=true&from_srp=true&qid=NAtwtTrIMc&rank=1".to_string())]);
    }
}