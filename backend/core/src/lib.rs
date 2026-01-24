use serde::{Serialize, Deserialize};


#[derive(thiserror::Error, Debug, Serialize)]
pub enum HistoryError {
    #[error("invalid range")]
    InvalidRange,
    #[error("record not found")]
    NotFound
}



pub async fn get_history(left: u64, right: u64) -> Result<Vec<Record>, HistoryError> {

    let h = vec![
        Record {
            ts: "2023-10-01T12:00:00Z".to_string(),
            locations: vec![
                Location {
                    lat: 40.7128,
                    lon: -74.0060,
                    ts: "2023-10-01T12:00:00Z".to_string(),
                    route: RouteInfo {
                        route_id: "1".to_string(),
                        route_short_name: "A".to_string(),
                        route_long_name: "Route A".to_string(),
                    },
                    vehicle_id: "V1".to_string(),
                },
                Location {
                    lat: 34.0522,
                    lon: -118.2437,
                    ts: "2023-10-01T12:05:00Z".to_string(),
                    route: RouteInfo {
                        route_id: "2".to_string(),
                        route_short_name: "B".to_string(),
                        route_long_name: "Route B".to_string(),
                    },
                    vehicle_id: "V2".to_string(),
                },
            ],
        },
        Record {
            ts: "2023-10-01T12:10:00Z".to_string(),
            locations: vec![
                Location {
                    lat: 41.8781,
                    lon: -87.6298,
                    ts: "2023-10-01T12:10:00Z".to_string(),
                    route: RouteInfo {
                        route_id: "3".to_string(),
                        route_short_name: "C".to_string(),
                        route_long_name: "Route C".to_string(),
                    },
                    vehicle_id: "V3".to_string(),
                },
                Location {
                    lat: 29.7604,
                    lon: -95.3698,
                    ts: "2023-10-01T12:15:00Z".to_string(),
                    route: RouteInfo {
                        route_id: "4".to_string(),
                        route_short_name: "D".to_string(),
                        route_long_name: "Route D".to_string(),
                    },
                    vehicle_id: "V4".to_string(),
                },
            ],
        },
    ];


    Ok(h)
}

#[derive(Serialize, Debug, Clone)]
pub struct Record {
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

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct RouteInfo {
    route_id: String,
    route_short_name: String,
    route_long_name: String
}


