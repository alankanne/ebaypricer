use clap::Parser;
use std::fs::File;
use csv::Writer;

mod auth;
mod search;

/// Program to recommend eBay pricing
#[derive(Parser, Debug)]
#[command(name = "ebay_pricer")]
#[command(about = "Fetches listings and suggests sale prices", long_about = None)]
struct Args {
    /// One or more eBay listing URLs
    links: Vec<String>,
}

fn extract_item_id(url: &str) -> Option<String> {
    if let Some(idx) = url.find("/itm/") {
        let after = &url[idx + 5..];
        let id_part = after
            .split(|c| c == '/' || c == '?')
            .next()
            .unwrap_or("");
        if !id_part.is_empty() {
            return Some(id_part.to_string());
        }
    }
    None
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    let token = auth::get_token()
        .await
        .expect("Failed to fetch eBay OAuth token");
    let client = reqwest::Client::new();

    for link in args.links {
        if let Some(item_id) = extract_item_id(&link) {
            println!("Legacy Item ID: {}", item_id);

            match search::get_item_by_legacy_id(&client, &token, &item_id).await {
                Ok(item) => {
                    let anchor_price: f64 = item
                        .price
                        .as_ref()
                        .and_then(|p| p.value.as_ref())
                        .and_then(|v| v.parse::<f64>().ok())
                        .unwrap_or(0.0);

                    let anchor_condition = item.condition.clone().unwrap_or("<none>".into());

                    println!("Anchor Title: {}", item.title.clone().unwrap_or("<none>".into()));
                    println!("Anchor Price: {}", anchor_price);
                    println!("Condition: {}", anchor_condition);

                    if anchor_price > 0.0 {
                        let lower = anchor_price * 0.7;
                        let upper = anchor_price * 1.3;

                        if let Some(query) = &item.title {
                            match search::search_by_keywords(&client, &token, query).await {
                                Ok(comps) => {
                                    let mut prices: Vec<f64> = comps
                                        .iter()
                                        .filter(|c| {
                                            c.condition
                                                .as_ref()
                                                .map(|cond| cond == &anchor_condition)
                                                .unwrap_or(false)
                                        })
                                        .filter_map(|c| {
                                            c.price
                                                .as_ref()
                                                .and_then(|p| p.value.as_ref())
                                                .and_then(|v| v.parse::<f64>().ok())
                                        })
                                        .filter(|&p| p >= lower && p <= upper)
                                        .collect();

                                    prices.sort_by(|a, b| a.partial_cmp(b).unwrap());

                                    if !prices.is_empty() {
                                        let mid = prices.len() / 2;
                                        let median = if prices.len() % 2 == 0 {
                                            (prices[mid - 1] + prices[mid]) / 2.0
                                        } else {
                                            prices[mid]
                                        };

                                        println!("--- Pricing Recommendations ---");
                                        println!("Fast sale (90%): {:.2}", median * 0.9);
                                        println!("Market (100%): {:.2}", median);
                                        println!("Patient (110%): {:.2}", median * 1.1);

                                        // Write comps to per-item CSV
                                        let filename = format!("comps_{}.csv", item_id.clone());
                                        let file = File::create(&filename)
                                            .expect("Unable to create CSV file");
                                        let mut wtr = Writer::from_writer(file);

                                        wtr.write_record(&[
                                            "AnchorID",
                                            "Title",
                                            "Price",
                                            "Condition",
                                            "URL",
                                        ])
                                        .unwrap();

                                        let mut comps_sorted: Vec<_> = comps
                                            .iter()
                                            .filter(|c| {
                                                c.condition
                                                    .as_ref()
                                                    .map(|cond| cond == &anchor_condition)
                                                    .unwrap_or(false)
                                            })
                                            .filter_map(|c| {
                                                c.price
                                                    .as_ref()
                                                    .and_then(|p| p.value.as_ref())
                                                    .and_then(|v| v.parse::<f64>().ok())
                                                    .map(|pr| (pr, c))
                                            })
                                            .filter(|(pr, _)| *pr >= lower && *pr <= upper)
                                            .collect();

                                        comps_sorted.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

                                        for (pr, c) in comps_sorted {
                                            wtr.write_record(&[
                                                item_id.clone(),
                                                c.title.clone().unwrap_or_default(),
                                                format!("{:.2}", pr),
                                                c.condition.clone().unwrap_or_default(),
                                                c.itemWebUrl.clone().unwrap_or_default(),
                                            ])
                                            .unwrap();
                                        }

                                        wtr.flush().unwrap();
                                        println!("Comps written to {}", filename);
                                    } else {
                                        println!(
                                            "No comps found in ±30% range with condition = {}.",
                                            anchor_condition
                                        );
                                    }
                                }
                                Err(e) => eprintln!("Search error: {}", e),
                            }
                        }
                    }
                }
                Err(e) => eprintln!("API error for {}: {}", item_id, e),
            }
        } else {
            eprintln!("Could not extract item ID from link: {}", link);
        }
    }
}
