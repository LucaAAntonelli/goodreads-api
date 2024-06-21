pub mod goodreads_api {
    // TODO: Change implementation of metadata loading
    // Possible options: threading, only load metadata of chosen book, ...
    use reqwest;
    use itertools::izip;

    use scraper::{Html, Selector};

    #[derive(Debug, PartialEq)]
    pub struct GoodreadsBook {
        
        title: String,
        authors: Vec<String>,
        pages: u64,
        series: Option<String>,
        url: String
    }

    fn extract_metadata_from_book_url(url: &str) -> (Vec<String>, u64, Option<String>) {
        let response = reqwest::blocking::get(url).unwrap().text().unwrap();
        let document = Html::parse_document(&response);

        let authors = vec![];
        let pages = 0;
        let series = Option::None;

        (authors, pages, series)

        
    }

    impl GoodreadsBook {

        pub fn new(title: String, authors: Vec<String>, pages: u64, series: Option<String>, url: String) -> Self {
            Self {title, authors, pages, series, url}
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
                
                // TODO: Change to account for multiple authors
                let authors_element = book_element.select(&authors_selector).next().expect("No authors found");

                let title_and_series = title_series_element.text().collect::<Vec<_>>().concat();
                let authors = authors_element.text().map(|x| x.to_string()).collect::<Vec<_>>();

                println!("Found title and series: {}, by {}", title_and_series, authors.concat());
                
                // for (a_element_title, a_element_authors) in izip!(tr_element.select(&a_selector_title), tr_element.select(&a_selector_authors)) {
                //     let title = a_element_title.value().attr("title").expect("No title found").to_string();
                //     println!("Found title: {title}");
                //     let url = "https://www.goodreads.com".to_string() + a_element_title.value().attr("href").expect("No url found");
                //     let mut authors: Vec<String> = Vec::new();
                //     for span_element in a_element_authors.select(&span_selector) {
                //         // Access and print the text content of the <span> element
                //         let author = span_element.text().collect::<Vec<_>>().concat();
                //         println!("Found author name: {}", author);
                //         authors.push(author);
                //     }
                //     // let (authors, pages, series) = extract_metadata_from_book_url(&url);
                //     let pages = 0;
                //     let series = Option::None;
                    
                //     books.push(GoodreadsBook::new(title, authors, pages, series, url));
                // }
            }
            books
        }
    }

       

}

mod tests {
    use super::goodreads_api;

    #[test]
    fn test_hobbit() {
        let books = goodreads_api::GoodreadsBook::search("The Hobbit");
        assert_eq!(books[0..1], vec![goodreads_api::GoodreadsBook::new("The Hobbit".to_string(), vec!["J.R.R. Tolkien".to_string()], 310, Option::None, "https://www.goodreads.com/book/show/5907.The_Hobbit?from_search=true&from_srp=true&qid=NAtwtTrIMc&rank=1".to_string())]);
    }

    #[test]
    fn test_neverwhere() {
        let books = goodreads_api::GoodreadsBook::search("Neverwhere");
        assert_eq!(books[0..1], vec![goodreads_api::GoodreadsBook::new("Neverwhere".to_string(), vec!["Neil Gaiman".to_string()], 370, Option::None, "https://www.goodreads.com/book/show/14497.Neverwhere?from_search=true&from_srp=true&qid=NAtwtTrIMc&rank=2".to_string())]);
    }
}