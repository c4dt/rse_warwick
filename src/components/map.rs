use dioxus::{logger::tracing, prelude::*};
use dioxus_leaflet::{Map, MapMarker, MapPosition};

struct _POI {
    latitude: f64,
    longitude: f64,
    name: &'static str,
}

const _POIS: [_POI; 5] = [
    _POI {
        latitude: 52.378933,
        longitude: -1.562204,
        name: "Let's not be stupid",
    },
    _POI {
        latitude: 52.379486,
        longitude: -1.562931,
        name: "Days of Judgement - Cat I",
    },
    _POI {
        latitude: 52.379046,
        longitude: -1.565627,
        name: "Song - Version V",
    },
    _POI {
        latitude: 52.375521,
        longitude: -1.565444,
        name: "Hare",
    },
    _POI {
        latitude: 52.379095,
        longitude: -1.561604,
        name: "Ripple Effect",
    },
];

#[component]
pub fn MapPOI() -> Element {
    // let a = use_signal(|| POI {
    //         latitude: 0,
    //         longitude: 0,
    //         name: "",
    //     });
    // a.read().
    #[cfg(feature = "web")]
    {
        use dioxus_sdk::geolocation::{init_geolocator, use_geolocation, PowerMode};
        use flarch::tasks::wait_ms;
        let geolocator = init_geolocator(PowerMode::High);
        let latest_coords_caller = use_geolocation();
        let latest_coords = match latest_coords_caller() {
            Ok(v) => v,
            Err(e) => {
                use dioxus::logger::tracing;

                tracing::error!("Initializing: {:?}", e);
                return rsx! {p{"Error"}};
            }
        };

        rsx!(
            div {
                style: "text-align: center;",
                h1 { "Warwick POIs - Collect 'em all!" }
                List{latitude: latest_coords.latitude, longitude: latest_coords.longitude}
                LocationTracker{latitude: latest_coords.latitude, longitude: latest_coords.longitude}
            }
        )
    }
    #[cfg(not(target_family = "wasm"))]
    rsx! {}
}

#[component]
fn List(latitude: f64, longitude: f64) -> Element {
    let mut dists: Vec<(usize, f64)> = _POIS
        .iter()
        .enumerate()
        .map(|(i, poi)| {
            (
                i,
                ((poi.latitude - latitude).powf(2.0) + (poi.longitude - longitude).powf(2.0))
                    .sqrt(),
            )
        })
        .collect();
    dists.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
    let closest = dists.first().unwrap();
    let pos = format!("{:.4}/{:.4}", latitude, longitude);
    rsx! {
        p { "Your position: {pos}" }
        p { "Closest POI {_POIS[closest.0].name} at {(closest.1 * 10000.).floor()}m" }

    }
}

#[component]
fn LocationTracker(latitude: f64, longitude: f64) -> Element {
    tracing::info!("{latitude} / {longitude}");
    let path_markers = _POIS
        .iter()
        .map(|poi| MapMarker {
            lat: poi.latitude,
            lng: poi.longitude,
            title: poi.name.into(),
            description: None,
            icon: None,
            popup_options: None,
            custom_data: None,
        })
        .collect();

    rsx! {
        Map {
            initial_position: MapPosition::new(latitude, longitude, 32.),
            markers: path_markers,
            height: "500px",
        }
    }
}
