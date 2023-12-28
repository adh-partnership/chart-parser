extern crate serde;
extern crate serde_xml_rs;

use serde::{Deserialize, Serialize};
use std::fs::File;

#[derive(Debug, Deserialize, Serialize, PartialEq)]
struct Airport {
    #[serde(rename = "ID")]
    id: String,
    #[serde(rename = "apt_ident")]
    apt_ident: Option<String>,
    #[serde(rename = "record")]
    records: Option<Vec<Chart>>,
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
struct Chart {
    #[serde(rename = "chart_code")]
    chart_code: String,
    #[serde(rename = "chart_name")]
    chart_name: String,
    #[serde(rename = "pdf_name")]
    pdf_name: String,
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
struct City {
    #[serde(rename = "ID")]
    id: String,
    #[serde(rename = "airport_name")]
    airports: Vec<Airport>,
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
struct State {
    #[serde(rename = "ID")]
    id: String,
    #[serde(rename = "city_name")]
    city_names: Vec<City>,
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
struct DigitalTpp {
    #[serde(rename = "state_code")]
    state_codes: Vec<State>,
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    let today = chrono::Local::now().format("%Y-%m-%d").to_string();

    println!("ADH Chart Parser v2");
    println!("Today is {}", today);
    println!(
        "Is today a cylce day? {}",
        faa::is_cycle_date(today.as_str())
    );
    if !faa::is_cycle_date(today.as_str()) {
        // Check if the FORCE environment variable is set
        if std::env::var("FORCE").is_err() {
            println!("Today is not a cycle day and FORCE is not set. Exiting.");
            std::process::exit(0);
        }

        println!("Today is not a cycle day but FORCE is set. Continuing.");
    }

    let table = if let Ok(db_table) = std::env::var("DB_TABLE") {
        db_table
    } else {
        "airport_charts".to_string()
    };

    if std::env::var("STATES").is_err() {
        println!("STATES environment variable not set. Exiting.");
        std::process::exit(1);
    }
    let states: Vec<String> = std::env::var("STATES")
        .unwrap()
        .split(',')
        .map(|s| s.to_string())
        .collect();
    println!("States: {:?}", states);

    let cycle = faa::calculate_cycle(&today).unwrap();
    println!("Cycle: {}", cycle);

    let (start_date, end_date) = faa::calculate_cycle_dates(&cycle);
    println!("Cycle start date: {}", start_date);
    println!("Cycle end date: {}", end_date);

    println!("Connecting to database...");
    let login = format!(
        "mysql://{}:{}@{}:{}/{}",
        std::env::var("DB_USER").unwrap(),
        std::env::var("DB_PASSWORD").unwrap(),
        std::env::var("DB_HOST").unwrap(),
        std::env::var("DB_PORT").unwrap(),
        std::env::var("DB_DATABASE").unwrap()
    );
    let pool = sqlx::mysql::MySqlPool::connect(login.as_str()).await?;
    println!("Connected to database.");

    println!("Getting d-tpp.xml");
    let body: String;
    // Check if file d-tpp.xml exists first
    if std::fs::metadata("d-tpp.xml").is_err() {
        let response = reqwest::get(format!(
            "https://aeronav.faa.gov/d-tpp/{}/xml_data/d-tpp_Metafile.xml",
            cycle
        ))
        .await?;
        if response.status() != 200 {
            println!("Failed to get d-tpp.xml");
            std::process::exit(1);
        }
        body = response.text().await?;
        // write body to d-tpp.xml
        std::fs::write("d-tpp.xml", body).unwrap();
    }

    println!("Parsing d-tpp.xml");
    parse_data(
        "d-tpp.xml",
        &states,
        &cycle,
        &start_date,
        &end_date,
        &pool,
        &table,
    )
    .await;

    println!("Cleaning up old charts");
    sqlx::query("DELETE FROM airport_charts WHERE cycle != ?")
        .bind(&cycle)
        .execute(&pool)
        .await?;

    println!("Done");

    Ok(())
}

async fn parse_data(
    file_path: &str,
    states: &[String],
    cycle: &str,
    start_date: &str,
    end_date: &str,
    pool: &sqlx::mysql::MySqlPool,
    table: &str,
) {
    let file = File::open(file_path).expect("Failed to open XML file");
    let digital_tpp: DigitalTpp = serde_xml_rs::from_reader(file).expect("Failed to parse XML");
    let all: String = "ALL".to_string();

    let mut chart_count = 0;
    let mut airport_count = 0;

    for state in digital_tpp.state_codes {
        for city in state.city_names {
            if states.contains(&state.id) || states.contains(&all) {
                for airport in city.airports {
                    // Check if airport has an apt_ident
                    if airport.apt_ident.is_none() {
                        println!("Airport has no apt_ident: {:?}", airport);
                        continue;
                    }

                    if airport.records.is_none() {
                        println!("Airport has no records: {:?}", airport);
                        continue;
                    }

                    airport_count += 1;
                    for chart in airport.records.unwrap() {
                        let chart_type = match chart.chart_code.as_str() {
                            "DP" | "STAR" | "IAP" => chart.chart_code.clone(),
                            _ => "OTHER".to_string(),
                        };

                        let apt_ident = airport.apt_ident.clone().unwrap();

                        update_chart(
                            &format!("FAA-{}-{}-{}", apt_ident, chart_type, chart.chart_name),
                            &apt_ident,
                            cycle,
                            start_date,
                            end_date,
                            &chart_type,
                            &chart.chart_name,
                            &format!("https://aeronav.faa.gov/d-tpp/{}/{}", cycle, chart.pdf_name),
                            pool,
                            &table,
                        )
                        .await;
                        chart_count += 1;
                    }
                }
            }
        }
    }

    println!(
        "Processed {} charts for {} airports",
        chart_count, airport_count
    );
}

#[allow(clippy::too_many_arguments)]
async fn update_chart(
    chart_id: &str,
    apt_ident: &str,
    cycle: &str,
    start_date: &str,
    end_date: &str,
    chart_type: &str,
    chart_name: &str,
    pdf_url: &str,
    pool: &sqlx::mysql::MySqlPool,
    table: &str,
) {
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
    .await
    .expect("Failed to update chart");
}
