pub mod goodreads_api {
    // TODO: Change implementation of metadata loading
    // Possible options: threading, only load metadata of chosen book, ...
    use reqwest;
    use regex::Regex;

    use scraper::{Html, Selector};

    #[derive(Debug)]
    pub struct GoodreadsBook {
        
        title: String,
        authors: Vec<String>,
        pages: u64,
        series: Option<String>,
        index: Option<f32>,
        url: String
    }

    fn extract_pages_from_url(url: &str) -> u64 {
        let response = reqwest::blocking::get(url).unwrap().text().unwrap();
        let document = Html::parse_document(&response);

        let pages_selector = Selector::parse("p[data-testid=pagesFormat]").unwrap();


        let pages = document
                            .select(&pages_selector)
                            .next()
                            .expect("No pages found")
                            .text()
                            .collect::<Vec<_>>()
                            .concat()
                            .trim()
                            .split_whitespace()
                            .next()
                            .expect("No pages found")
                            .parse::<u64>()
                            .unwrap_or_default();

        pages

        
    }

    fn split(title_and_series: &str) -> Result<(String, Option<String>, Option<f32>), &'static str> {
        let re = Regex::new(r"^(.*)\s\((.*),\s#(\d+)\)$").unwrap();

        if let Some(captures) = re.captures(title_and_series) {
            let title = captures.get(1).unwrap().as_str().to_string();
            let series = captures.get(2).unwrap().as_str().to_string();
            let index = captures.get(3).unwrap().as_str().parse::<f32>().unwrap();
            Ok((title, Some(series), Some(index)))
        } else {
            Ok((title_and_series.to_string(), None, None))
        }
    }

    impl GoodreadsBook {

        pub fn new(title: String, authors: Vec<String>, pages: u64, series: Option<String>, index: Option<f32>, url: String) -> Self {
            Self {title, authors, pages, series, index, url}
        }

        

        pub fn search(query: &str) -> Vec<Self> {

            let url = format!("https://www.goodreads.com/search?q={}&tab=books", query);
            
            let response = reqwest::blocking::get(&url).unwrap().text().unwrap();

            let document = Html::parse_document(&response);

            let book_selector = Selector::parse(r#"tr[itemscope][itemtype="http://schema.org/Book"]"#).unwrap();

            let title_series_selector = Selector::parse("a[class=bookTitle]").unwrap();

            let authors_selector = Selector::parse("a[class=authorName]").unwrap();

            let mut books = vec![];

            for book_element in document.select(&book_selector) {

                let title_series_element = book_element.select(&title_series_selector).next().expect("No title found");
                
                
                let authors_elements = book_element.select(&authors_selector).collect::<Vec<_>>();
                

                let title_and_series = title_series_element.text().collect::<Vec<_>>().concat().trim().to_string();
                let url = format!("https://www.goodreads.com{}", title_series_element.value().attr("href").expect("No URL found"));

                let (title, series, index) = split(&title_and_series).unwrap();

                let authors = authors_elements.iter().map(|x| x.text().collect::<Vec<_>>().concat()).collect::<Vec<_>>();

                let pages = extract_pages_from_url(&url);
                
                books.push(Self::new(title, authors, pages, series, index, url));
            }
            books
        }
    }

    impl PartialEq for GoodreadsBook {
        fn eq(&self, other: &Self) -> bool {
            self.title == other.title && self.authors == other.authors && self.pages == other.pages && self.series == other.series && self.index == other.index
        }
    }

}

mod tests {
    use crate::goodreads_api::GoodreadsBook;

    use super::goodreads_api;

    

    #[test]
    fn test_hobbit() {
        let books = goodreads_api::GoodreadsBook::search("The Hobbit");
        assert_eq!(books[0], goodreads_api::GoodreadsBook::new("The Hobbit".to_string(), 
                                                                vec!["J.R.R. Tolkien".to_string()], 
                                                                366, 
                                                                Option::Some("The Lord of the Rings".to_string()), 
                                                                Option::Some(0.0), 
                                                                "https://www.goodreads.com/book/show/5907.The_Hobbit?from_search=true&from_srp=true&qid=NAtwtTrIMc&rank=1".to_string()));
    }

    #[test]
    fn test_neverwhere() {
        let books = goodreads_api::GoodreadsBook::search("Neverwhere");
        assert_eq!(books[0], goodreads_api::GoodreadsBook::new("Neverwhere".to_string(), 
                                                                vec!["Neil Gaiman".to_string()], 
                                                                370, 
                                                                Option::Some("London Below".to_string()), 
                                                                Option::Some(1.0), 
                                                                "https://www.goodreads.com/book/show/14497.Neverwhere?from_search=true&from_srp=true&qid=NAtwtTrIMc&rank=2".to_string()));
    }
}