pub mod goodreads_api {
    use reqwest::get;
    use scraper::{Html, Selector};

    pub async fn search(query: &str) {
        let response = get(format!("https://www.goodreads.com/search?q={}&qid=", query.replace(" ", "+"))).await.expect("Could not send request");


        let body = response.text().await.expect("Could not read response");

        let url_vec = extract_href(&body);

        let document = Html::parse_document(&body);

        let title_selector = Selector::parse(".bookTitle").expect("Failed to parse CSS selector");
        for (element, url) in document.select(&title_selector).into_iter().zip(url_vec) {
            let title = element.text().collect::<Vec<_>>().join("");
            println!("Found book {} with URL {}", title, url);
        }
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

    #[test]
    fn it_works() {
        block_on(goodreads_api::search("Dresden Files"));
    }
}
