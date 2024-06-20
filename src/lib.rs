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

            // TODO: Select individual books via table tag <tr>, then title and possible series from <a class="bookTitle">
            // and authors from <a class="authorName">, prevents issues with selecting simpy by <a> etc.

            let tr_selector = Selector::parse(r#"tr[itemscope][itemtype="http://schema.org/Book"]"#).unwrap();

            let a_selector_title = Selector::parse("a:not([class])").unwrap();

            let a_selector_authors = Selector::parse(r#"a.authorName"#).unwrap();

            let span_selector = Selector::parse(r#"span[itemprop="name"]"#).unwrap();

            let mut books = vec![];

            for tr_element in document.select(&tr_selector) {
                
                for (a_element_title, a_element_authors) in izip!(tr_element.select(&a_selector_title), tr_element.select(&a_selector_authors)) {
                    let title = a_element_title.value().attr("title").expect("No title found").to_string();
                    println!("Found title: {title}");
                    let url = "https://www.goodreads.com".to_string() + a_element_title.value().attr("href").expect("No url found");
                    let mut authors: Vec<String> = Vec::new();
                    for span_element in a_element_authors.select(&span_selector) {
                        // Access and print the text content of the <span> element
                        let author = span_element.text().collect::<Vec<_>>().concat();
                        println!("Found author name: {}", author);
                        authors.push(author);
                    }
                    // let (authors, pages, series) = extract_metadata_from_book_url(&url);
                    let pages = 0;
                    let series = Option::None;
                    
                    books.push(GoodreadsBook::new(title, authors, pages, series, url));
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