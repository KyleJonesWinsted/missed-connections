use std::{
    env::args,
    fmt::Debug,
    time::{Instant, SystemTime, UNIX_EPOCH},
};

use reqwest::blocking;

fn main() {
    let start = Instant::now();
    let url = "https://svc.metrotransit.org/mtgtfs/tripupdates.pb";
    let bytes = blocking::get(url).unwrap().bytes().unwrap();
    let data: gtfs_realtime::FeedMessage = prost::Message::decode(bytes.as_ref()).unwrap();
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;
    let past_departures = data
        .entity
        .iter()
        .filter_map(|entity| {
            entity.trip_update.as_ref().map(|tu| {
                tu.stop_time_update
                    .iter()
                    .filter(|stu| {
                        stu.departure
                            .and_then(|d| d.time)
                            .map(|t| t < now)
                            .unwrap_or(false)
                    })
                    .map(|stu| PastDeparture {
                        trip_id: tu.trip.trip_id(),
                        route: tu.trip.route_id(),
                        stop_id: stu.stop_id(),
                        time: stu.departure.map(|d| d.time()).unwrap_or_default(),
                    })
            })
        })
        .flatten();

    let db = rusqlite::Connection::open(args().last().unwrap()).unwrap();

    let mut insert = db
        .prepare(
            "INSERT OR REPLACE INTO 
                departure (trip_id, time, route_id, stop_id) 
                VALUES (:1, :2, :3, :4)",
        )
        .unwrap();

    let mut count = 0;
    for departure in past_departures {
        count += insert
            .execute((
                departure.trip_id,
                departure.time,
                departure.route,
                departure.stop_id,
            ))
            .unwrap();
    }
    println!(
        "found {} past departures. updated DB. {:?}",
        count,
        start.elapsed()
    );
}

#[derive(Debug)]
struct PastDeparture<'a> {
    trip_id: &'a str,
    time: i64,
    route: &'a str,
    stop_id: &'a str,
}
