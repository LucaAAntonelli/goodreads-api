pub mod goodreads_api {
    use reqwest::get;
    use scraper::{Html, Selector};

    #[derive(Debug, PartialEq)]
    pub struct Book {
        title: String,
        authors: Vec<String>,
        pages: u32,
        series: Option<String>
    }

    impl Book {
        pub async fn new(url: String) -> Self {
            let body = get(url).await.expect("Could not send request").text().await.expect("Could not read response");
            
            let doc = Html::parse_document(&body);

            let title_selector = Selector::parse("h1.Text.Text__title1[data-testid=bookTitle]").expect("Failed to find object '.Text_title1'!");
            let title = doc.select(&title_selector).next().expect("No matching element").text().collect::<Vec<_>>().join("");

            let author_selector = Selector::parse("span.ContributorLink__name[data-testid=name]").expect("Failed to find object");
            let mut authors = vec![];
            for auth in doc.select(&author_selector) {
                let author = auth.text().collect::<Vec<_>>().join("");
                authors.push(author);
            }
            authors.sort();
            authors.dedup();

            let pages_selector = Selector::parse("p[data-testid=pagesFormat]").expect("Error trying to select pages");

            let page_str = doc.select(&pages_selector).next().expect("asdf").text().collect::<Vec<_>>().join("");
            
            let pages: u32 = page_str.split(" ").collect::<Vec<_>>()[0].parse().unwrap();
            
            let series_selector = Selector::parse("a[aria-label^=Book]").expect("Error creating series selector");
            
            let series = match doc.select(&series_selector).next() {
                None => Option::None,
                Some(val) => Some(val.text().collect::<Vec<_>>().join("").split("#").collect::<Vec<_>>()[0].trim().to_owned())
            };


            Self {title, authors, pages, series}
        }

        pub fn create_book_by_hand(title: String, authors: Vec<String>, pages: u32, series: Option<String>) -> Self {
            Self{title, authors, pages, series}
        }
    }

    pub async fn search(query: &str) {
        let response = get(format!("https://www.goodreads.com/search?q={}&qid=", query.replace(" ", "+"))).await.expect("Could not send request");


        let body = response.text().await.expect("Could not read response");

        let url_vec = extract_href(&body);

        let document = Html::parse_document(&body);

        let title_selector = Selector::parse(".bookTitle").expect("Failed to parse CSS selector");
        let book_title = document.select(&title_selector).into_iter().next().unwrap().text().collect::<Vec<_>>().join("");
        let book_url = format!("https://goodreads.com{}", url_vec[0]);
        println!("Found book {book_title} at {book_url}");
    }

    fn extract_href(html: &str) -> Vec<String> {
        let document = Html::parse_document(html);
        let selector = Selector::parse("a.bookTitle").unwrap();
    
        let mut url_vec = Vec::new();
        for element in document.select(&selector) {
            let href = element.value().attr("href");
            url_vec.extend(href.map(|s| s.to_string()));
        }
        
    
        url_vec
    }

}


#[cfg(test)]
mod tests {

    use super::*;
    use tokio_test::block_on;
    use super::goodreads_api::Book;

    #[test]
    fn it_works() {
        block_on(goodreads_api::search("White Night dresden files"));
    }

    #[test]
    fn test_book() {
         let result = block_on(Book::new(String::from("https://www.goodreads.com/book/show/47212.Storm_Front?from_search=true&from_srp=true&qid=5OiExORxlI&rank=1")));

         assert_eq!(result, Book::create_book_by_hand("Storm Front".to_owned(), vec!["Jim Butcher".to_owned()], 355 as u32, Some("The Dresden Files".to_owned())));
    }

    #[test]
    fn test_multiple_authors() {
        let result = block_on(Book::new(String::from("https://www.goodreads.com/book/show/7743175-a-memory-of-light?from_search=true&from_srp=true&qid=kWxTLnUHTj&rank=1")));
        assert_eq!(result, Book::create_book_by_hand("A Memory of Light".to_owned(), vec!["Brandon Sanderson".to_owned(), "Robert Jordan".to_owned()], 912 as u32, Some("The Wheel of Time".to_owned())));
    }

    #[test]
    fn test_no_series() {
        let result = block_on(Book::new(String::from("https://www.goodreads.com/book/show/61439040-1984?from_search=true&from_srp=true&qid=52Ze8HuhoQ&rank=1")));
        assert_eq!(result, Book::create_book_by_hand("1984".to_owned(), vec!["George Orwell".to_owned(), "Thomas Pynchon".to_owned()], 368 as u32, None));

    

    }
}
