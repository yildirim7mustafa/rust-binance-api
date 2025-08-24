#[macro_use] extern crate rocket;

use rocket::serde::{Serialize, Deserialize, json::Json};
use rocket::http::Status;

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
/*
#[get("/coins")]
fn coins() -> Json<Vec<Coin>> {
    Json(get_coins())
}
*/

#[get("/coins")]
async fn coins() -> Result<Json<Vec<Coin>>, Status> {
    match get_coins().await {
        Ok(list) => Ok(Json(list)),
        Err(_) => Err(Status::InternalServerError),
    }
}

/* 
fn get_coins() -> Vec<Coin> {
    let coin1 = Coin { name: "BTC".to_string(), price: 123.123 };
    let coin2 = Coin { name: "UTC".to_string(), price: 5.55 };
    vec![coin1, coin2]
}
*/

async fn get_coins() -> Result<Vec<Coin>, Box<dyn std::error::Error>> {
    let url = "https://api.binance.com/api/v3/ticker/price";
    let resp = reqwest::get(url).await?.json::<Vec<Ticker>>().await?;

    let mut coins: Vec<Coin> = resp
        .into_iter()
        // Sadece USDT paritelerini al (BTCUSDT, ETHUSDT, ...)
        .filter(|t| t.symbol.ends_with("USDT"))
        // İstersen kaldıraçlı tokenleri atla (UP/DOWN)
        .filter(|t| !t.symbol.contains("UP") && !t.symbol.contains("DOWN"))
        .map(|t| {
            let name = t.symbol.trim_end_matches("USDT").to_string();
            let price = t.price.parse::<f64>().unwrap_or(0.0);
            Coin { name, price }
        })
        .collect();

    // (Opsiyonel) İsme göre sırala
    coins.sort_by(|a, b| a.name.cmp(&b.name));

    Ok(coins)
}

#[launch]
fn rocket() -> _ {
    rocket::build().mount("/", routes![coins])
    // İstersen köke almak için:
    // rocket::build().mount("/", routes![index, coins])
}
