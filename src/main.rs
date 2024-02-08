use reqwest::{self, Response};
use select::document::Document;
use select::predicate::Name;
use serde::Deserialize;
use std::error::Error;
use tokio;
use url::Url;

#[derive(Deserialize, Debug)]
struct SearchResult {
    search_metadata: SearchMetadata,
    search_parameters: SearchParameters,
    search_information: SearchInformation,
    ads: Option<Vec<Ad>>,
    organic_results: Option<Vec<OrganicResult>>,
}

#[derive(Deserialize, Debug)]
struct SearchMetadata {
    id: String,
    status: String,
    json_endpoint: String,
    created_at: String,
    processed_at: String,
    duckduckgo_url: String,
    raw_html_file: String,
    prettify_html_file: String,
    total_time_taken: f64,
}

#[derive(Deserialize, Debug)]
struct SearchParameters {
    engine: String,
    q: String,
    kl: String,
}

#[derive(Deserialize, Debug)]
struct SearchInformation {
    organic_results_state: String,
}

#[derive(Deserialize, Debug)]
struct Ad {
    position: i32,
    title: String,
    link: String,
    source: String,
    snippet: String,
    sitelinks: Option<Vec<Sitelink>>,
}

#[derive(Deserialize, Debug)]
struct OrganicResult {
    position: i32,
    title: String,
    link: String,
    snippet: String,
    favicon: String,
    sitelinks: Option<Vec<Sitelink>>,
}

#[derive(Deserialize, Debug)]
struct Sitelink {
    title: String,
    link: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let response =
        reqwest::get("https://serpapi.com/search?engine=duckduckgo&q=apple+inc&kl=us-en&api_key=2c535d256c51683d08c1cc62bb02ef8cf4ed7d051a7ad87337843ee300e0db36").await?;

    if response.status().is_success() {
        let search_result: SearchResult = response.json().await?;

        println!("Search Metadata: {:#?}", search_result.search_metadata);
        println!("Search Results: {:#?}", search_result.organic_results);
        // create a vec of links from the search results
        let links: Vec<String> = search_result
            .organic_results
            .unwrap()
            .iter()
            .map(|result| result.link.clone())
            .collect();

        let mut tasks = vec![];

        for link in links {
            if let Ok(_absolute_link) = Url::parse(&link) {
                let task = tokio::spawn(async move {
                    let response = reqwest::get(link)
                        .await
                        .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send>)?;
                    let body = response
                        .text()
                        .await
                        .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send>)?;

                    let document = Document::from(body.as_str());
                    for node in document.find(Name("a")).filter_map(|n| n.attr("href")) {
                        println!("{}", node);
                    }

                    Ok::<(), Box<dyn Error + Send>>(())
                });

                tasks.push(task);
            }
        }

        for task in tasks {
            let _ = task.await?;
        }

        return Ok(());
    } else {
        println!("Request failed with status: {}", response.status());
    }
    Ok(())
}
