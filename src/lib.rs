pub mod goodreads_api {

use core::fmt;
use std::fmt::{Display, Formatter};
use reqwest::Client;
use regex::Regex;
use scraper::{Html, Selector};
use log::{info, error};
use itertools::{izip, Itertools};

#[derive(Clone, Debug, PartialEq)]
pub struct SeriesInfo {
    name: String,
    volume: f32,
}

impl SeriesInfo {
    pub fn new(name: String, volume: f32) -> Self {
        Self {name, volume}
    }

    pub fn name(&self) -> String {
        self.name.clone()
    }

    pub fn volume(&self) -> f32 {
        self.volume
    }
}

#[derive(Clone, Debug)]
pub struct GoodreadsBook {
    title: String,
    authors: Vec<String>,
    pages: u64,
    series_info: Option<Vec<SeriesInfo>>,
    url: String,
    cover_image: Option<String>,
}

impl GoodreadsBook {
    pub fn new(
        title: String,
        authors: Vec<String>,
        pages: u64,
        series_info: Option<Vec<SeriesInfo>>,
        url: String,
        cover_image: Option<String>,
    ) -> Self {
        Self {
            title,
            authors,
            pages,
            series_info,
            url,
            cover_image,
        }
    }

    pub fn title(&self) -> String {
        self.title.clone()
    }

    pub fn authors(&self) -> Vec<String> {
        self.authors.clone()
    }

    pub fn pages(&self) -> u64 {
        self.pages
    }

    pub fn series_info(&self) -> Option<Vec<SeriesInfo>> {
        self.series_info.clone()
    
    }

    pub fn url(&self) -> String {
        self.url.clone()
    }

    pub fn cover_image(&self) -> Option<String> {
        self.cover_image.clone()
    }

    pub async fn search(query: &str) -> Vec<Self> {
        let url = format!("https://www.goodreads.com/search?q={}&tab=books", query);
        info!("Sending request to URL: {}", url);
    
        let client = Client::builder().user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:105.0) Gecko/20100101").build().expect("Failed to build reqwest client");
        let response = client.get(&url)
            .send()
            .await
            .unwrap_or_else(|err| {
                error!("Failed to send request: {}", err);
                panic!("Failed to send request to Goodreads API");
            })
            .text()
            .await
            .unwrap_or_else(|err| {
                error!("Failed to parse response: {}", err);
                panic!("Failed to parse response from Goodreads API");
            });
        info!("Received response: {}", response);
    
        let document = Html::parse_document(&response);
        let book_selector = Selector::parse(r#"tr[itemscope][itemtype="http://schema.org/Book"]"#).unwrap();
        let title_series_selector = Selector::parse("a[class=bookTitle]").unwrap();
        let authors_selector = Selector::parse("a[class=authorName]").unwrap();
        let cover_image_selector = Selector::parse("img[class=bookCover]").unwrap();
        let mut books = vec![];
        info!("Collecting URLs to all books' pages");
        let urls = document.select(&book_selector).map(|e| 
            format!("https://www.goodreads.com{}", e.select(&title_series_selector)
                                                .next()
                                                .expect("No title found").value().attr("href").expect("No URL found")
                                                )).collect::<Vec<String>>();
        
        info!("Collected all URLs");
        let mut book_webpages = vec![]; 
        for url in &urls {
            info!("Collecting response for url {url}");
            let response = client.get(url).send().await.unwrap().text().await.unwrap();
            book_webpages.push(response);
        }

        for (book_element, book_webpage, url) in izip!(document.select(&book_selector), book_webpages, urls) {
            info!("Processing book");
            let title_series_element = book_element.select(&title_series_selector).next().expect("No title found");
            let authors_elements = book_element.select(&authors_selector).collect::<Vec<_>>();
            let title_and_series = title_series_element
                .text()
                .collect::<Vec<_>>()
                .concat()
                .trim()
                .to_string();
            
            let cover_image = book_element.select(&cover_image_selector).next().map(|x| sanitize_url(x.value().attr("src").expect("Couldn't parse src to &str!")));
            info!("Processed cover image");
            let (title, series_info) = split(&title_and_series);

            info!("Processed title, series and index");
            let authors = authors_elements
                .iter()
                .map(|x| x.text().collect::<Vec<_>>().concat())
                .collect::<Vec<_>>();
            info!("Processed authors");
            let pages = extract_pages_from_url(book_webpage);
            info!("Processed number of pages");
            //let pages = 0;             
            info!("Adding {title} by {} to vector",  &authors.join(", "));
            books.push(Self::new(title, authors, pages, series_info, url, cover_image));
        }
        books
    }
}

pub fn extract_pages_from_url(response: String) -> u64 {
    info!("Extracting pages from book site");
    
    let document = Html::parse_document(&response);
    let pages_selector = Selector::parse("p[data-testid=pagesFormat]").unwrap();

    if let Some(pages) = document.select(&pages_selector).next() {
        pages
            .text()
            .collect::<Vec<_>>()
            .concat()
            .trim()
            .split_whitespace()
            .next()
            .expect("No pages found")
            .parse()
            .unwrap_or(0)
    } else {
        0
    }
}

pub fn split(title_and_series: &str) -> (String, Option<Vec<SeriesInfo>>) {
    info!("Splitting String into title, series and index");
    let re = Regex::new(r"^(.*)\s\((.*)\)$").unwrap();
    let series_re = Regex::new(r"([^#]+)#(\d+)").unwrap();
    
    if let Some(captures) = re.captures(title_and_series) {
        let mut series_info_vec = vec![];
        let title = captures.get(1).unwrap().as_str().to_string();
        let series_info_string = captures.get(2).unwrap().as_str();
        for series_cap in series_re.captures_iter(series_info_string) {
            let series_name = series_cap.get(1).unwrap().as_str().trim().replace(",", "").replace(";", "").trim().to_string();
            let volume = series_cap.get(2).unwrap().as_str().parse::<f32>().unwrap();
            series_info_vec.push(SeriesInfo::new(series_name, volume));
        }
        if series_info_vec.len() > 0 {
            (title, Some(series_info_vec))
        } else{
            (title_and_series.to_owned(), None)
        }
    } else {
        (title_and_series.to_owned(), None)
    }
}

fn sanitize_url(dirty_url: &str) -> String {
    info!("Sanitizing cover image URL");
    // Remove sequence of "._XXXX_" from all URLs where X is any alphanumerical value
    let re = regex::Regex::new(r"\._[a-zA-Z0-9]{4}_").expect("Failed to build regex pattern");
    re.replace(dirty_url, "").to_string()
}

impl PartialEq for GoodreadsBook {
    fn eq(&self, other: &Self) -> bool {
        self.title == other.title
            && self.authors == other.authors
            && self.pages == other.pages
            && self.series_info == other.series_info
    }
}

impl Display for GoodreadsBook {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(
            f,
            "{} by {}, {} pages, series: {}",
            self.title,
            self.authors.join(", "),
            self.pages,
            match &self.series_info {
                Some(series_info) => series_info.iter().map(|x| format!("{}, book #{}", x.name(), x.volume())).join(", "),
                None => "None".to_owned()
            }
        )
    }
}
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::goodreads_api::GoodreadsBook;
    use goodreads_api::SeriesInfo;
    use tokio::runtime::Runtime;

    #[test]
    fn test_hobbit() {
        // Create a tokio runtime
        let rt = Runtime::new().unwrap();

        // Run the test asynchronously
        rt.block_on(async {
            let books = GoodreadsBook::search("The Hobbit").await;
            assert_eq!(
                books[0],
                GoodreadsBook::new(
                    "The Hobbit".to_string(),
                    vec!["J.R.R. Tolkien".to_string()],
                    366,
                    Some(vec![SeriesInfo::new("The Lord of the Rings".to_string(), 0.0)]),
                    "https://www.goodreads.com/book/show/5907.The_Hobbit?from_search=true&from_srp=true&qid=NAtwtTrIMc&rank=1".to_string(),
                    Option::None,
                )
            );
        });
    }

    #[test]
    fn test_neverwhere() {
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            let books = GoodreadsBook::search("Neverwhere").await;
            assert_eq!(
                books[0],
                GoodreadsBook::new(
                    "Neverwhere".to_string(),
                    vec!["Neil Gaiman".to_string()],
                    370,
                    Some(vec![SeriesInfo::new("London Below".to_string(), 1.0)]),
                    "https://www.goodreads.com/book/show/14497.Neverwhere?from_search=true&from_srp=true&qid=NAtwtTrIMc&rank=2".to_string(),
                    Option::None
                )
            );
        });
    }

    #[test]
    fn test_neverwhere_full() {
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            let books = GoodreadsBook::search("Neverwhere Neil Gaiman").await;
            assert_eq!(
                books[0],
                GoodreadsBook::new(
                    "Study Guide: Neverwhere by Neil Gaiman".to_string(),
                    vec!["SuperSummary".to_string()],
                    46,
                    None,
                    "https://www.goodreads.com/book/show/14497.Neverwhere?from_search=true&from_srp=true&qid=NAtwtTrIMc&rank=2".to_string(),
                    Option::None
                )
            );
        });
    }

    #[test]
    fn test_bedlam() {
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            let books = GoodreadsBook::search("Bedlam Derek Landy").await;
            assert_eq!(
                books[0],
                GoodreadsBook::new(
                    "Bedlam".to_string(),
                    vec!["Derek Landy".to_string()],
                    608,
                    Some(vec![SeriesInfo::new("Skulduggery Pleasant".to_string(), 12.0)]),
                    "https://www.goodreads.com/book/show/135390.Bedlam?from_search=true&from_srp=true&qid=NAtwtTrIMc&rank=3".to_string(),
                    Option::None
                )
            );
        });
    }

    #[test]
    fn test_title_series_volume_splitter() {
        assert_eq!(("Neverwhere".to_owned(), Some(vec![SeriesInfo::new("London Below".to_string(), 1.0)])), goodreads_api::split("Neverwhere (London Below, #1)"));
    }

    #[test]
    fn test_multiple_series() {
        assert_eq!(
            ("The Colour of Magic".to_owned(), 
            Some(vec![
                SeriesInfo::new("Discworld".to_owned(), 1.0),
                SeriesInfo::new("Rincewind".to_owned(), 1.0)
            ])), 
            goodreads_api::split("The Colour of Magic (Discworld #1; Rincewind #1)"));
    }
}
