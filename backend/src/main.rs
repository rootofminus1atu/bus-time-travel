use std::{collections::HashMap, sync::Arc};
use tokio::sync::Mutex;
use axum::{response::IntoResponse, routing::get, Extension, Json, Router};
use dotenvy::dotenv;
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use chrono::Utc;
use tower_http::cors::{CorsLayer, Any};
use std::sync::LazyLock;


type GenericError = Box<dyn std::error::Error + Send + Sync + 'static>;

type LocationHistory = Arc<Mutex<Vec<Record>>>;


pub static ROUTES: LazyLock<HashMap<String, RouteInfo>> = LazyLock::new(|| {
    let contents = std::fs::read_to_string("assets/routes.txt").expect("Failed to read routes file");
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(true)
        .from_reader(contents.as_bytes());

    rdr.records()
        .filter_map(|result| {
            result.ok().and_then(|rec| {
                let route_id = rec.get(0)?.to_string();
                let short_name = rec.get(2)?.to_string();
                let long_name = rec.get(3)?.to_string();
                Some(RouteInfo { route_id, short_name, long_name })
            })
        })
        .map(|r| (r.route_id.clone(), r))
        .collect()
});

#[tokio::main]
async fn main() -> Result<(), GenericError> {
    println!("Hello, world!");
    dotenv().ok();

    LazyLock::force(&ROUTES);


    let api_key = std::env::var("API_KEY").expect("no api key found");
    let client = ClientWithKeys::new(api_key.clone(), api_key);
    let history: LocationHistory = Arc::new(Mutex::new(Vec::new()));

    let client_clone = client.clone();
    let history_clone = history.clone();

    tokio::spawn(async move {
        loop {
            match get_location(&client_clone).await {
                Ok(record) => {
                    let mut list = history_clone.lock().await;
                    list.push(record);

                    let max_list_len = 720;
                    let list_len = list.len();

                    if list.len() > max_list_len {
                        list.drain(0..(list_len - max_list_len));
                    }
                },
                Err(e) => {
                    println!("ERROR: {}\n{:?}", e, e);
                }
            }
            println!("eeping");
            tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;
        }
    });

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let router = Router::new()
        .route("/current", get(get_current))
        .route("/history", get(get_history))
        .layer(cors)
        .layer(Extension(history))
        .layer(Extension(client));

    let listener = tokio::net::TcpListener::bind("localhost:3000").await.unwrap();
    println!("listening on 3000");
    axum::serve(listener, router).await.unwrap();

    Ok(())
}

async fn get_current(Extension(client): Extension<ClientWithKeys>) -> Result<impl IntoResponse, Error> {
    get_location(&client).await.map(Json)
}

async fn get_history(Extension(history): Extension<LocationHistory>) -> Result<impl IntoResponse, Error> {
    let list = history.lock().await.clone();
    Ok(Json(list))
}

async fn get_location(client: &ClientWithKeys) -> Result<Record, Error> {
    let res = client.client.get("https://api.nationaltransport.ie/gtfsr/v2/Vehicles?format=json")
        .header("x-api-key", client.bus_api_key.as_str())
        .send()
        .await?
        .json::<serde_json::Value>()
        .await?;

    if let Some(status) = res.get("statusCode").and_then(|v| v.as_u64()) {
        if status == 429 {
            let msg = res.get("message").and_then(|m| m.as_str()).unwrap_or("Rate limit exceeded").to_string();
            return Err(Error::RateLimited(msg));
        }
    }

    let pretty = serde_json::to_string_pretty(&res).unwrap_or_default();
    for (i, line) in pretty.lines().take(50).enumerate() {
        println!("{:02}: {}", i + 1, line);
    }
    let res: Res = serde_json::from_value(res)?;


    let route_ids = ["4658_98094", "4658_98097", "4658_98087", "4658_98244", "4685_99482"];
    
    // let entity = res.entity.iter().find(|e| e.vehicle.trip.route_id == route_id).expect("damn it doesnt exist wtf");
    // dbg!(&entity);
    
    let locations = res.entity.iter()
        .filter(|e| route_ids.contains(&e.vehicle.trip.route_id.as_str()))
        .map(|e| Location {
            lat: e.vehicle.position.latitude, 
            lon: e.vehicle.position.longitude,
            ts: e.vehicle.timestamp.clone(),
            route: ROUTES.get(&e.vehicle.trip.route_id).unwrap().clone(),
            vehicle_id: e.vehicle.vehicle.id.clone()
        })
        .collect::<Vec<_>>();

    for loc in &locations {
        let link = format!("https://www.google.com/maps?q={},{}", loc.lat, loc.lon);
        println!("FOLLOW THE LINK: {}", link);
    }

    let record = Record {
        ts: Utc::now().timestamp().to_string(),
        locations
    };

    Ok(record)
}



#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Some error")]
    SomeError,
    #[error("Cat reqwest error: {0}")]
    ReqwestError(#[from] reqwest::Error),
    #[error(transparent)]
    SerdeJsonError(#[from] serde_json::Error),
    #[error("Rate limited: {0}")]
    RateLimited(String),
}

impl IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        println!("->> {}", self);

        let body = Json(serde_json::json!({
            "error": format!("{}", &self)
        }));

        let status_code = match &self {
            _ => StatusCode::INTERNAL_SERVER_ERROR
        };
        
        (status_code, body).into_response()
    }
}




#[derive(Debug, Clone)]
pub struct ClientWithKeys {
    client: reqwest::Client,
    cat_api_key: Arc<String>,
    bus_api_key: Arc<String>
}

impl ClientWithKeys {
    pub fn new_w_client(client: reqwest::Client, cat_api_key: String, bus_api_key: String) -> Self {
        Self {
            client,
            cat_api_key: Arc::new(cat_api_key),
            bus_api_key: Arc::new(bus_api_key)
        }
    }

    pub fn new(cat_api_key: String, bus_api_key: String) -> Self {
        Self::new_w_client(reqwest::Client::new(), cat_api_key, bus_api_key)
    }
}



#[derive(Serialize, Debug, Clone)]
struct Record {
    ts: String,
    locations: Vec<Location>
}


#[derive(Serialize, Debug, Clone)] 
struct Location {
    lat: f64,
    lon: f64,
    ts: String,
    route: RouteInfo,
    vehicle_id: String
}



#[derive(Deserialize, Debug, Clone)]
struct Res {
    entity: Vec<Entity>
}

#[derive(Deserialize, Debug, Clone)]
struct Entity {
    id: String,
    vehicle: Vehicle
}

#[derive(Deserialize, Debug, Clone)]
struct Vehicle {
    trip: Trip,
    timestamp: String,
    position: Position,
    vehicle: VehicleDetails
}

#[derive(Deserialize, Debug, Clone)]
struct VehicleDetails {
    id: String
}

#[derive(Deserialize, Debug, Clone)]
struct Trip {
    route_id: String
}

#[derive(Deserialize, Debug, Clone)]
struct Position {
    latitude: f64,
    longitude: f64
}

#[derive(Serialize, Debug, Clone)]
pub struct RouteInfo {
    route_id: String,
    short_name: String,
    long_name: String
}
