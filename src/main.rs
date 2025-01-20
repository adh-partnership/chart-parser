use anyhow::Result;
use chrono::NaiveDate;
use dotenv::dotenv;
use faa::DATE_FORMAT;
use log::{debug, error, info, warn};
use models::DigitalTpp;
use sqlx::mysql::MySqlPool;
use std::{env, fs, process};

mod faa;
mod models;

/// Downloaded FAA file name.
const DOWNLOAD_FILE_NAME: &str = "d-tpp.xml";

#[tokio::main]
async fn main() {
    // env load and logger setup
    if let Err(e) = dotenv() {
        eprintln!("Could not load .env file: {e}");
        process::exit(1);
    }
    if env::var("RUST_LOG").is_err() {
        env::set_var("RUST_LOG", "info");
    }
    pretty_env_logger::init();

    // basic info
    let today = chrono::Local::now().format("%Y-%m-%d").to_string();
    info!("ADH Chart Parser v2");
    info!("Today is {}", today);

    let today_is = faa::is_cycle_date(&today);
    info!("Is today a cycle day? {}", today_is);
    if !today_is && env::var("FORCE").is_err() {
        warn!("Today is not a cycle day and FORCE is not set. Exiting.");
        return;
    }

    let table = match env::var("DB_TABLE") {
        Ok(t) => t,
        Err(_) => String::from("airport_charts"),
    };
    let states: Vec<String> = env::var("STATES")
        .expect("Missing \"STATES\" env var")
        .split(',')
        .map(|s| s.to_string())
        .collect();
    info!("States: {:?}", states);

    let cycle = match faa::get_cycle_for(&today) {
        Some(c) => c,
        None => {
            error!("Could not determine cycle for \"{today}\"");
            process::exit(1);
        }
    };
    info!(
        "Cycle start date is {}, code {}",
        cycle.start.to_string(),
        cycle.code
    );

    info!("Connecting to database...");
    let login = format!(
        "mysql://{}:{}@{}:{}/{}",
        std::env::var("DB_USER").unwrap(),
        std::env::var("DB_PASSWORD").unwrap(),
        std::env::var("DB_HOST").unwrap(),
        std::env::var("DB_PORT").unwrap(),
        std::env::var("DB_DATABASE").unwrap()
    );
    let pool = match sqlx::mysql::MySqlPool::connect(login.as_str()).await {
        Ok(p) => p,
        Err(e) => {
            error!("Could not connect to the database: {e}");
            process::exit(1);
        }
    };
    info!("Connected to database.");

    info!("Getting {DOWNLOAD_FILE_NAME}");
    if fs::metadata("d-tpp.xml").is_err() {
        let resp = reqwest::get(format!(
            "https://aeronav.faa.gov/d-tpp/{}/xml_data/d-tpp_Metafile.xml",
            cycle.code
        ))
        .await
        .expect("Could not send request");
        if !resp.status().is_success() {
            error!("Got status code {} from FAA site", resp.status().as_u16());
            process::exit(1);
        }
        let body = match resp.text().await {
            Ok(b) => b,
            Err(e) => {
                error!("Could not read response body: {e}");
                process::exit(1)
            }
        };
        if let Err(e) = fs::write(DOWNLOAD_FILE_NAME, body) {
            error!("Error writing response body to file: {e}");
            process::exit(1);
        }
    } else {
        debug!("{DOWNLOAD_FILE_NAME} already exists; not re-downloading");
    }

    info!("Parsing {DOWNLOAD_FILE_NAME}");
    if let Err(e) = parse_data(&states, cycle.code, &cycle.start, &cycle.end, &pool, &table).await {
        error!("Could not parse the document: {e}");
        process::exit(1);
    }

    info!("Cleaning up old charts");
    if let Err(e) = sqlx::query(&format!("DELETE FROM {} WHERE cycle != ?", table))
        .bind(cycle.code)
        .execute(&pool)
        .await
    {
        error!("Could not execute SQL to clean up old charts: {e}");
        process::exit(1);
    }

    info!("Done");
}

/// Load the downloaded file, parse, and update the database.
async fn parse_data(
    states: &[String],
    cycle: u16,
    start_date: &NaiveDate,
    end_date: &NaiveDate,
    pool: &MySqlPool,
    table: &str,
) -> Result<()> {
    let file_content = fs::read_to_string(DOWNLOAD_FILE_NAME)?;
    let digital_tpp: DigitalTpp = serde_xml_rs::from_str(&file_content)?;
    let all = String::from("ALL");

    let mut chart_count = 0usize;
    let mut airport_count = 0usize;

    for state in digital_tpp.state_codes {
        for city in state.city_names {
            if states.contains(&state.id) || states.contains(&all) {
                for airport in city.airports {
                    if airport.apt_ident.is_none() {
                        warn!("Airport {} has no apt_ident", airport.id);
                        continue;
                    }
                    let records = match airport.records {
                        Some(r) => r,
                        None => {
                            warn!("Airport {} has no records", airport.id);
                            continue;
                        }
                    };
                    airport_count += 1;
                    for chart in records {
                        let chart_type = match chart.chart_code.as_str() {
                            "DP" | "STAR" | "IAP" => chart.chart_code.clone(),
                            _ => String::from("OTHER"),
                        };
                        let apt_ident = airport.apt_ident.as_ref().unwrap().clone();
                        update_chart(
                            &format!("FAA-{}-{}-{}", apt_ident, chart_type, chart.chart_name),
                            &apt_ident,
                            cycle,
                            &start_date.format(DATE_FORMAT).to_string(),
                            &end_date.format(DATE_FORMAT).to_string(),
                            &chart_type,
                            &chart.chart_name,
                            &format!("https://aeronav.faa.gov/d-tpp/{}/{}", cycle, chart.pdf_name),
                            pool,
                            table,
                        )
                        .await?;
                        chart_count += 1;
                    }
                }
            }
        }
    }

    info!("Processed {chart_count} chartts for {airport_count} airports");
    Ok(())
}

/// Update the chart in the database.
#[allow(clippy::too_many_arguments)]
async fn update_chart(
    chart_id: &str,
    apt_ident: &str,
    cycle: u16,
    start_date: &str,
    end_date: &str,
    chart_type: &str,
    chart_name: &str,
    pdf_url: &str,
    pool: &MySqlPool,
    table: &str,
) -> Result<()> {
    sqlx::query(&format!(
        "INSERT INTO {} (id, airport_id, cycle, from_date, to_date, chart_code, chart_name, chart_url)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?)
        ON DUPLICATE KEY UPDATE
            cycle = VALUES(cycle),
            from_date = VALUES(from_date),
            to_date = VALUES(to_date),
            chart_code = VALUES(chart_code),
            chart_name = VALUES(chart_name),
            chart_url = VALUES(chart_url)",
        table
    ))
        .bind(chart_id)
        .bind(apt_ident)
        .bind(cycle)
        .bind(start_date)
        .bind(end_date)
        .bind(chart_type)
        .bind(chart_name)
        .bind(pdf_url)
        .execute(pool)
        .await?;
    Ok(())
}
