pub mod goodreads_api {
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

            let mut books = vec![];

            for tr_element in document.select(&tr_selector) {
                
                for a_element in tr_element.select(&a_selector) {
                    let title = a_element.value().attr("title").expect("No title found").to_string();
                    let url = a_element.value().attr("href").expect("No url found").to_string();
                    books.push(GoodreadsBook::new(title, vec![], 0, Option::None, url));
                }
            }
            books
        }
    }

       

}

mod tests {
    use goodreads_api::GoodreadsBook;

    use super::*;

    #[test]
    fn test_hobbit() {
        let books = goodreads_api::GoodreadsBook::search("The Hobbit");
        assert_eq!(books[0..1], vec![GoodreadsBook::new("The Hobbit".to_string(), vec!["J.R.R. Tolkien".to_string()], 310, Option::None, "https://www.goodreads.com/book/show/5907.The_Hobbit?from_search=true&from_srp=true&qid=NAtwtTrIMc&rank=1".to_string())]);
    }

    #[test]
    fn test_neverwhere() {
        let books = goodreads_api::GoodreadsBook::search("Neverwhere");
        assert_eq!(books[0..1], vec![GoodreadsBook::new("Neverwhere".to_string(), vec!["Neil Gaiman".to_string()], 370, Option::None, "https://www.goodreads.com/book/show/14497.Neverwhere?from_search=true&from_srp=true&qid=NAtwtTrIMc&rank=2".to_string())]);
    }
}