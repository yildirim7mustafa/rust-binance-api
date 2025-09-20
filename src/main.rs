#[macro_use] extern crate rocket;

use rocket::serde::{Serialize, Deserialize, json::Json};
use rocket::http::Status;
use lapin::{options::*, types::FieldTable, BasicProperties, Connection, ConnectionProperties, message::Delivery};
use futures_util::StreamExt;
use tokio_amqp::*;
use anyhow::Result;
use prometheus::{
    Encoder, TextEncoder,
    IntCounter, IntGauge, Histogram,
    register_int_counter, register_int_gauge, register_histogram,
};
use lazy_static::lazy_static;
use rocket::response::content::RawText;

lazy_static! {
    static ref HTTP_REQ_TOTAL: IntCounter =
        register_int_counter!("http_requests_total", "Total HTTP requests").unwrap();
    static ref HTTP_REQ_ERRORS: IntCounter =
        register_int_counter!("http_requests_errors_total", "Total HTTP request errors").unwrap();
    static ref COINS_COUNT: IntGauge =
        register_int_gauge!("coins_last_count", "Number of coins in last response").unwrap();
    static ref REQ_DURATION: Histogram =
        register_histogram!("http_request_duration_seconds", "Request duration seconds").unwrap();
}

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
struct Coin {
    name: String,
    price: f64,
}

// Binance dan donen deger ticker/price 
#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
struct Ticker {
    symbol: String,
    price: String,
}

#[get("/metrics")]
fn metrics() -> RawText<String> {
    let mut buf = Vec::new();
    let enc = TextEncoder::new();
    enc.encode(&prometheus::gather(), &mut buf).unwrap();
    RawText(String::from_utf8(buf).unwrap())
}

async fn publish_to_queue(coins: &Vec<Coin>) -> Result<(), Box<dyn std::error::Error>> {
    // broker adresi (compose’da "rabbitmq" servisi)
    let addr = "amqp://guest:guest@rabbitmq:5672/%2f";
    let conn = Connection::connect(addr, ConnectionProperties::default().with_tokio()).await?;
    let channel = conn.create_channel().await?;

    channel
        .queue_declare(
            "coins_queue",
            QueueDeclareOptions::default(),
            FieldTable::default(),
        )
        .await?;

    let payload = serde_json::to_vec(&coins)?;
    channel
        .basic_publish(
            "",
            "coins_queue",
            BasicPublishOptions::default(),
            &payload,
            BasicProperties::default(),
        )
        .await?
        .await?;

    println!("Published {} coins to RabbitMQ!", coins.len());
    Ok(())
}

async fn get_coins() -> Result<Vec<Coin>> {
    let url = "https://api.binance.com/api/v3/ticker/price";
    let resp = reqwest::get(url).await?.json::<Vec<Ticker>>().await?;

    let mut coins: Vec<Coin> = resp
        .into_iter()
        .filter(|t| t.symbol.ends_with("USDT"))
        .filter(|t| !t.symbol.contains("UP") && !t.symbol.contains("DOWN"))
        .map(|t| {
            let name = t.symbol.trim_end_matches("USDT").to_string();
            let price = t.price.parse::<f64>().unwrap_or(0.0);
            Coin { name, price }
        })
        .collect();

    coins.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(coins)
}

#[get("/coins")]
async fn coins() -> Result<Json<Vec<Coin>>, Status> {
    let _timer = REQ_DURATION.start_timer(); // süre ölç
    HTTP_REQ_TOTAL.inc();                    // istek sayacı

    match get_coins().await {
        Ok(list) => {
            COINS_COUNT.set(list.len() as i64); // kaç coin döndük

            // --- RabbitMQ producer (senin kodun) ---
            if let Err(e) = publish_to_queue(&list).await {
                // metrikte hata say
                HTTP_REQ_ERRORS.inc();
                eprintln!("RabbitMQ publish error: {:?}", e);
                // İstersen burada 500 döndürmek yerine sadece log/metrikleyip devam edebiliriz.
            }

            Ok(Json(list))
        }
        Err(_) => {
            HTTP_REQ_ERRORS.inc();
            Err(Status::InternalServerError)
        }
    }
}

async fn consume_queue() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "amqp://guest:guest@rabbitmq:5672/%2f";
    let conn = Connection::connect(addr, ConnectionProperties::default().with_tokio()).await?;
    let channel = conn.create_channel().await?;

    channel
        .queue_declare(
            "coins_queue",
            QueueDeclareOptions::default(),
            FieldTable::default(),
        )
        .await?;

    let mut consumer = channel
        .basic_consume(
            "coins_queue",
            "my_consumer",
            BasicConsumeOptions::default(),
            FieldTable::default(),
        )
        .await?;

    tokio::spawn(async move {
        while let Some(result) = consumer.next().await {
            match result {
                Ok(delivery) => handle_message(delivery).await,
                Err(err) => eprintln!("Consumer error: {:?}", err),
            }
        }
    });

    Ok(())
}

async fn handle_message(delivery: Delivery) {
    if let Ok(msg) = String::from_utf8(delivery.data.clone()) {
        println!("Consumed message: {}", msg);
    }
    delivery.ack(BasicAckOptions::default()).await.expect("ack");
}



#[rocket::main]
async fn main() -> Result<(), rocket::Error> {
    // RabbitMQ consumer’ı arka planda başlat
    rocket::tokio::spawn(async {
        if let Err(e) = consume_queue().await {
            eprintln!("Consumer init error: {e:?}");
        }
    });

    rocket::build()
        .mount("/", routes![coins,metrics])
        .launch()
        .await?;

    Ok(())
}
