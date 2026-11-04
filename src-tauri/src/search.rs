use crate::ssrf::validate_external_url;
use regex::Regex;
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub title: String,
    pub snippet: String,
    pub url: String,
    pub source: Option<String>,
}

/// Executes a multi-source search combining primary search engine + Google News RSS + Hacker News concurrently.
pub async fn execute_multi_source_search(
    query: &str,
    engine: &str,
    searx_url: Option<&str>,
    brave_key: Option<&str>,
    domain_terms: &[String],
    excluded_terms: &[String],
) -> Vec<SearchResult> {
    let start = std::time::Instant::now();
    let mut all_results = Vec::new();
    let mut seen_urls = std::collections::HashSet::new();

    let q_lower = query.to_lowercase();
    let is_tech = q_lower.contains("software")
        || q_lower.contains("engine")
        || q_lower.contains("ai")
        || q_lower.contains("code")
        || q_lower.contains("game")
        || q_lower.contains("rust")
        || q_lower.contains("python")
        || q_lower.contains("tech");

    // Run primary search, Google News RSS, and Hacker News concurrently in parallel
    let (primary_res, news_res, hn_res) = tokio::join!(
        execute_search(query, engine, searx_url, brave_key),
        fetch_google_news_rss(query),
        async {
            if is_tech {
                fetch_hacker_news(query).await
            } else {
                Ok(Vec::new())
            }
        }
    );

    let mut primary_count = 0;
    if let Ok(primary_results) = primary_res {
        for r in primary_results {
            if seen_urls.insert(r.url.clone()) {
                primary_count += 1;
                all_results.push(r);
            }
        }
    }

    let mut news_count = 0;
    if let Ok(news_results) = news_res {
        for r in news_results {
            if seen_urls.insert(r.url.clone()) {
                news_count += 1;
                all_results.push(r);
            }
        }
    }

    let mut hn_count = 0;
    if let Ok(hn_results) = hn_res {
        for r in hn_results {
            if seen_urls.insert(r.url.clone()) {
                hn_count += 1;
                all_results.push(r);
            }
        }
    }

    let raw_count = all_results.len();
    let filtered = filter_by_domain_relevance(all_results, domain_terms, excluded_terms);
    let elapsed_ms = start.elapsed().as_millis() as u64;

    crate::logger::log_event(
        None,
        "SEARCH",
        elapsed_ms,
        &format!(
            "Multi-source search for \"{}\" yielded {} items ({} filtered to {}) in {}ms [primary: {}, news: {}, hn: {}]",
            query,
            raw_count,
            raw_count,
            filtered.len(),
            elapsed_ms,
            primary_count,
            news_count,
            hn_count
        ),
    );

    filtered
}

/// Executes a single search across the selected backend.
pub async fn execute_search(
    query: &str,
    engine: &str,
    searx_url: Option<&str>,
    brave_key: Option<&str>,
) -> Result<Vec<SearchResult>, String> {
    match engine.to_lowercase().as_str() {
        "searxng" => search_searxng(query, searx_url.unwrap_or("http://localhost:8080")).await,
        "brave search" | "brave" => search_brave(query, brave_key.unwrap_or("")).await,
        _ => search_duckduckgo(query).await,
    }
}

/// Google News RSS Fetcher: High-reputation journalistic articles without bot blockers.
pub async fn fetch_google_news_rss(query: &str) -> Result<Vec<SearchResult>, String> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(6))
        .build()
        .map_err(|e| e.to_string())?;

    let encoded_query = url::form_urlencoded::byte_serialize(query.as_bytes()).collect::<String>();
    let feed_url = format!(
        "https://news.google.com/rss/search?q={}&hl=en-US&gl=US&ceid=US:en",
        encoded_query
    );

    let resp = client
        .get(&feed_url)
        .header(
            "User-Agent",
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/124.0.0.0 Safari/537.36",
        )
        .send()
        .await
        .map_err(|e| format!("Google News RSS error: {}", e))?;

    let xml = resp.text().await.map_err(|e| e.to_string())?;
    parse_rss_xml(&xml)
}

/// Parses RSS XML items into clean SearchResult objects.
pub fn parse_rss_xml(xml: &str) -> Result<Vec<SearchResult>, String> {
    let item_re = Regex::new(r"(?s)<item>(.*?)</item>").map_err(|e| e.to_string())?;
    let title_re = Regex::new(r"(?s)<title>(.*?)</title>").map_err(|e| e.to_string())?;
    let link_re = Regex::new(r"(?s)<link>(.*?)</link>").map_err(|e| e.to_string())?;
    let desc_re = Regex::new(r"(?s)<description>(.*?)</description>").map_err(|e| e.to_string())?;
    let source_re = Regex::new(r#"(?s)<source[^>]*>(.*?)</source>"#).map_err(|e| e.to_string())?;

    let html_tag_re = Regex::new(r"<[^>]+>").map_err(|e| e.to_string())?;

    let mut results = Vec::new();

    for cap in item_re.captures_iter(xml).take(8) {
        let item_body = &cap[1];

        let title = title_re
            .captures(item_body)
            .map(|c| clean_xml_entities(&c[1]))
            .unwrap_or_default();

        let link = link_re
            .captures(item_body)
            .map(|c| c[1].trim().to_string())
            .unwrap_or_default();

        let raw_desc = desc_re
            .captures(item_body)
            .map(|c| c[1].to_string())
            .unwrap_or_default();

        let clean_desc = html_tag_re.replace_all(&raw_desc, " ");
        let snippet = clean_xml_entities(&clean_desc);

        let source = source_re
            .captures(item_body)
            .map(|c| clean_xml_entities(&c[1]))
            .or_else(|| {
                if let Some(dash_idx) = title.rfind(" - ") {
                    Some(title[dash_idx + 3..].trim().to_string())
                } else {
                    None
                }
            });

        if !title.is_empty() && !link.is_empty() {
            results.push(SearchResult {
                title,
                snippet,
                url: link,
                source,
            });
        }
    }

    Ok(results)
}

fn clean_xml_entities(s: &str) -> String {
    s.replace("&amp;", "&")
        .replace("&quot;", "\"")
        .replace("&apos;", "'")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&#39;", "'")
        .replace("&nbsp;", " ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

/// Hacker News Algolia Search API
pub async fn fetch_hacker_news(query: &str) -> Result<Vec<SearchResult>, String> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(6))
        .build()
        .map_err(|e| e.to_string())?;

    let encoded_query = url::form_urlencoded::byte_serialize(query.as_bytes()).collect::<String>();
    let url = format!(
        "https://hn.algolia.com/api/v1/search?query={}&tags=story&hitsPerPage=6",
        encoded_query
    );

    let resp = client
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("Hacker News error: {}", e))?;

    let json: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;
    let mut results = Vec::new();

    if let Some(hits) = json["hits"].as_array() {
        for hit in hits {
            let title = hit["title"].as_str().unwrap_or_default().to_string();
            let points = hit["points"].as_i64().unwrap_or(0);
            let comments = hit["num_comments"].as_i64().unwrap_or(0);
            let object_id = hit["objectID"].as_str().unwrap_or_default();

            let target_url = hit["url"]
                .as_str()
                .filter(|u| !u.is_empty())
                .map(|u| u.to_string())
                .unwrap_or_else(|| format!("https://news.ycombinator.com/item?id={}", object_id));

            let snippet = format!(
                "Hacker News discussion ({} points, {} comments): {}",
                points, comments, title
            );

            if !title.is_empty() {
                results.push(SearchResult {
                    title,
                    snippet,
                    url: target_url,
                    source: Some("Hacker News".into()),
                });
            }
        }
    }

    Ok(results)
}

/// Filters results to ensure semantic relevance to the query domain
pub fn filter_by_domain_relevance(
    results: Vec<SearchResult>,
    domain_terms: &[String],
    excluded_terms: &[String],
) -> Vec<SearchResult> {
    if domain_terms.is_empty() && excluded_terms.is_empty() {
        return results;
    }

    let mut filtered = Vec::new();

    for r in results {
        let combined = format!("{} {}", r.title, r.snippet).to_lowercase();

        // Check if item contains an explicit exclusion (e.g. "textile ministry", "typing test")
        let mut is_excluded = false;
        for excl in excluded_terms {
            let excl_lower = excl.to_lowercase();
            if !excl_lower.is_empty() && combined.contains(&excl_lower) {
                is_excluded = true;
                break;
            }
        }

        if is_excluded {
            continue;
        }

        // Check if item matches at least one domain term (if domain terms specified)
        if !domain_terms.is_empty() {
            let has_domain_match = domain_terms.iter().any(|term| {
                let term_lower = term.to_lowercase();
                !term_lower.is_empty() && combined.contains(&term_lower)
            });

            if !has_domain_match {
                continue;
            }
        }

        filtered.push(r);
    }

    filtered
}

/// Ranks search results by relevance to the query and domain keywords,
/// takes strictly the top `limit` items, and compacts snippets to prevent context overflow.
pub fn rank_and_select_top_evidence(
    results: Vec<SearchResult>,
    query: &str,
    domain_keywords: &[String],
    limit: usize,
) -> Vec<SearchResult> {
    if results.is_empty() {
        return Vec::new();
    }

    let stop_words: std::collections::HashSet<&str> = [
        "a", "an", "the", "in", "on", "at", "to", "for", "of", "and", "or", "by", "with", "is",
        "are", "was", "were", "what", "which", "who", "when", "where", "why", "how", "top", "best",
    ]
    .iter()
    .cloned()
    .collect();

    let query_terms: Vec<String> = query
        .split_whitespace()
        .map(|w| w.to_lowercase())
        .map(|w| w.trim_matches(|c: char| !c.is_alphanumeric()).to_string())
        .filter(|w| !w.is_empty() && !stop_words.contains(w.as_str()))
        .collect();

    let mut scored: Vec<(f32, SearchResult)> = Vec::new();
    let mut seen_titles = std::collections::HashSet::new();

    for r in results {
        let title_clean = r.title.to_lowercase();
        let snippet_clean = r.snippet.to_lowercase();

        // Dedup nearly identical titles
        let title_key: String = title_clean
            .chars()
            .filter(|c| c.is_alphanumeric())
            .take(30)
            .collect();
        if !seen_titles.insert(title_key) {
            continue;
        }

        let mut score: f32 = 0.0;

        // Query term matching in Title & Snippet
        for qt in &query_terms {
            if title_clean.contains(qt) {
                score += 5.0;
            }
            if snippet_clean.contains(qt) {
                score += 2.0;
            }
        }

        // Domain keyword matching in Title & Snippet
        for dk in domain_keywords {
            let dk_lower = dk.to_lowercase();
            if !dk_lower.is_empty() {
                if title_clean.contains(&dk_lower) {
                    score += 4.0;
                }
                if snippet_clean.contains(&dk_lower) {
                    score += 1.5;
                }
            }
        }

        // Authority / Source boost
        if let Some(src) = &r.source {
            let src_lower = src.to_lowercase();
            if src_lower.contains("google news")
                || src_lower.contains("ign")
                || src_lower.contains("chronicle")
                || src_lower.contains("vgc")
                || src_lower.contains("eurogamer")
                || src_lower.contains("gamesradar")
            {
                score += 2.0;
            }
        }

        if r.snippet.len() < 20 {
            score -= 1.0;
        }

        scored.push((score, r));
    }

    // Sort descending by score
    scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));

    let top_items: Vec<SearchResult> = scored
        .into_iter()
        .take(limit)
        .map(|(_, mut r)| {
            // Compact snippet to ~120 words max to avoid context bloat
            let words: Vec<&str> = r.snippet.split_whitespace().collect();
            if words.len() > 120 {
                r.snippet = format!("{}...", words[..120].join(" "));
            }
            r
        })
        .collect();

    top_items
}

/// DuckDuckGo direct HTML search with CSS extraction
async fn search_duckduckgo(query: &str) -> Result<Vec<SearchResult>, String> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(6))
        .build()
        .map_err(|e| format!("HTTP error: {}", e))?;

    let form_params = [("q", query), ("b", "")];
    let resp = client
        .post("https://html.duckduckgo.com/html/")
        .header(
            "User-Agent",
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/124.0.0.0 Safari/537.36",
        )
        .form(&form_params)
        .send()
        .await
        .map_err(|e| format!("DuckDuckGo request error: {}", e))?;

    let html_content = resp.text().await.map_err(|e| e.to_string())?;
    let document = Html::parse_document(&html_content);

    let result_selector = Selector::parse(".result").map_err(|e| format!("{:?}", e))?;
    let title_selector = Selector::parse(".result__title a").map_err(|e| format!("{:?}", e))?;
    let snippet_selector = Selector::parse(".result__snippet").map_err(|e| format!("{:?}", e))?;

    let mut results = Vec::new();
    for element in document.select(&result_selector).take(6) {
        let title = element
            .select(&title_selector)
            .next()
            .map(|el| el.text().collect::<Vec<_>>().join(" ").trim().to_string())
            .unwrap_or_default();

        let raw_url = element
            .select(&title_selector)
            .next()
            .and_then(|el| el.value().attr("href"))
            .unwrap_or_default();

        let url = clean_ddg_url(raw_url);

        let snippet = element
            .select(&snippet_selector)
            .next()
            .map(|el| el.text().collect::<Vec<_>>().join(" ").trim().to_string())
            .unwrap_or_default();

        if !title.is_empty() && !url.is_empty() {
            results.push(SearchResult {
                title,
                snippet,
                url,
                source: Some("DuckDuckGo".into()),
            });
        }
    }

    Ok(results)
}

fn clean_ddg_url(raw_url: &str) -> String {
    if let Some(pos) = raw_url.find("uddg=") {
        let encoded = &raw_url[pos + 5..];
        let end_pos = encoded.find('&').unwrap_or(encoded.len());
        let slice = &encoded[..end_pos];
        if let Ok(decoded) = urlencoding_decode(slice) {
            return decoded;
        }
    }
    raw_url.to_string()
}

fn urlencoding_decode(s: &str) -> Result<String, String> {
    url::form_urlencoded::parse(s.as_bytes())
        .next()
        .map(|(k, _)| k.into_owned())
        .ok_or_else(|| "Failed to decode".to_string())
}

/// SearXNG JSON API search
async fn search_searxng(query: &str, searx_url: &str) -> Result<Vec<SearchResult>, String> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(6))
        .build()
        .map_err(|e| e.to_string())?;

    let url = format!(
        "{}/search?q={}&format=json",
        searx_url.trim_end_matches('/'),
        url::form_urlencoded::byte_serialize(query.as_bytes()).collect::<String>()
    );
    let resp = client
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("SearXNG error: {}", e))?;

    let json: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;
    let mut results = Vec::new();
    if let Some(items) = json["results"].as_array() {
        for it in items.iter().take(6) {
            results.push(SearchResult {
                title: it["title"].as_str().unwrap_or_default().to_string(),
                snippet: it["content"].as_str().unwrap_or_default().to_string(),
                url: it["url"].as_str().unwrap_or_default().to_string(),
                source: Some("SearXNG".into()),
            });
        }
    }
    Ok(results)
}

/// Brave Search API
async fn search_brave(query: &str, api_key: &str) -> Result<Vec<SearchResult>, String> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(6))
        .build()
        .map_err(|e| e.to_string())?;

    let resp = client
        .get("https://api.search.brave.com/res/v1/web/search")
        .query(&[("q", query), ("count", "6")])
        .header("X-Subscription-Token", api_key.trim())
        .header("Accept", "application/json")
        .send()
        .await
        .map_err(|e| format!("Brave error: {}", e))?;

    let json: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;
    let mut results = Vec::new();
    if let Some(items) = json["web"]["results"].as_array() {
        for it in items.iter().take(6) {
            results.push(SearchResult {
                title: it["title"].as_str().unwrap_or_default().to_string(),
                snippet: it["description"].as_str().unwrap_or_default().to_string(),
                url: it["url"].as_str().unwrap_or_default().to_string(),
                source: Some("Brave".into()),
            });
        }
    }
    Ok(results)
}

/// Fetches a webpage securely and cleans HTML down to readable text (max 8,000 chars).
pub async fn fetch_clean_webpage(url_str: &str) -> Result<String, String> {
    let safe_url = validate_external_url(url_str)?;

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(12))
        .build()
        .map_err(|e| e.to_string())?;

    let resp = client
        .get(safe_url.as_str())
        .header(
            "User-Agent",
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/124.0.0.0 Safari/537.36",
        )
        .send()
        .await
        .map_err(|e| format!("Visit failed: {}", e))?;

    let html = resp.text().await.map_err(|e| e.to_string())?;
    let document = Html::parse_document(&html);

    let body_selector = Selector::parse("body").map_err(|e| format!("{:?}", e))?;
    let text = if let Some(body) = document.select(&body_selector).next() {
        body.text().collect::<Vec<_>>().join(" ")
    } else {
        html
    };

    let cleaned: String = text.split_whitespace().collect::<Vec<_>>().join(" ");
    let max_len = 8000;
    if cleaned.len() > max_len {
        Ok(cleaned[..max_len].to_string())
    } else {
        Ok(cleaned)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_rss_xml() {
        let sample_xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<rss version="2.0"><channel>
<item>
    <title>Report: Next Xbox Targeting 2028 - Video Games Chronicle</title>
    <link>https://news.google.com/articles/12345</link>
    <description>&lt;a href="..."&gt;Full details on 2028 roadmap&lt;/a&gt;</description>
    <source url="https://www.videogameschronicle.com">Video Games Chronicle</source>
</item>
</channel></rss>"#;

        let items = parse_rss_xml(sample_xml).unwrap();
        assert_eq!(items.len(), 1);
        assert!(items[0].title.contains("Next Xbox Targeting 2028"));
        assert_eq!(items[0].url, "https://news.google.com/articles/12345");
        assert_eq!(items[0].source.as_deref(), Some("Video Games Chronicle"));
    }

    #[test]
    fn test_filter_by_domain_relevance() {
        let results = vec![
            SearchResult {
                title: "Top Anticipated Video Games in 2028".into(),
                snippet: "Comprehensive list of RPGs and action games planned for 2028".into(),
                url: "https://example.com/games".into(),
                source: None,
            },
            SearchResult {
                title: "Annual Report - Ministry of Textiles 2028".into(),
                snippet: "Export statistics for cotton and yarn in India".into(),
                url: "https://textiles.gov.in/report".into(),
                source: None,
            },
            SearchResult {
                title: "Class 11 Physical Education Games Chapter".into(),
                snippet: "Athletics, rules of soccer, and sports curriculum".into(),
                url: "https://pe.edu/ch11".into(),
                source: None,
            },
        ];

        let domain_terms = vec!["game".into(), "games".into(), "rpg".into(), "playstation".into()];
        let excluded_terms = vec!["textiles".into(), "physical education".into(), "cotton".into()];

        let filtered = filter_by_domain_relevance(results, &domain_terms, &excluded_terms);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].title, "Top Anticipated Video Games in 2028");
    }

    #[test]
    fn test_rank_and_select_top_evidence() {
        let results = vec![
            SearchResult {
                title: "Random unrelated post".into(),
                snippet: "Some random discussion about coffee".into(),
                url: "https://example.com/coffee".into(),
                source: None,
            },
            SearchResult {
                title: "Most anticipated upcoming video games targeting 2028".into(),
                snippet: "The Witcher 4 and Fallout 5 have targeting windows around 2028 according to studio roadmap disclosures.".into(),
                url: "https://ign.com/2028-games".into(),
                source: Some("IGN".into()),
            },
            SearchResult {
                title: "Upcoming 2028 video game releases and projections".into(),
                snippet: "Release calendar for games scheduled in 2028.".into(),
                url: "https://gamesradar.com/2028".into(),
                source: Some("GamesRadar".into()),
            },
        ];

        let domain_terms = vec!["video games".into(), "game".into(), "studio".into()];
        let top = rank_and_select_top_evidence(results, "upcoming games in 2028", &domain_terms, 2);

        assert_eq!(top.len(), 2);
        assert!(top[0].title.contains("2028"));
        assert!(top[0].source.is_some());
    }
}
