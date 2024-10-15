

CREATE TABLE IF NOT EXISTS departure (
    trip_id TEXT NOT NULL,
    time INTEGER NOT NULL,
    route_id TEXT NOT NULL,
    stop_id TEXT NOT NULL,
    UNIQUE(trip_id, route_id, stop_id)
);



