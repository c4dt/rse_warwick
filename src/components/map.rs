use dioxus::prelude::*;

struct POI {
    latitude: f64,
    longitude: f64,
    name: &'static str,
}

const POIS: [POI; 5] = [
    POI {
        latitude: 52.378933,
        longitude: -1.562204,
        name: "Let's not be stupid",
    },
    POI {
        latitude: 52.379486,
        longitude: -1.562931,
        name: "Days of Judgement - Cat I",
    },
    POI {
        latitude: 52.379046,
        longitude: -1.565627,
        name: "Song - Version V",
    },
    POI {
        latitude: 52.375521,
        longitude: -1.565444,
        name: "Hare",
    },
    POI {
        latitude: 52.379095,
        longitude: -1.561604,
        name: "Ripple Effect",
    },
];

#[component]
pub fn Map() -> Element {
    #[cfg(target_family = "wasm")]
    {
        use dioxus_sdk::geolocation::{init_geolocator, use_geolocation, PowerMode};
        let geolocator = init_geolocator(PowerMode::High);
        // let initial_coords = use_resource(move || async move {
        //     geolocator
        //         .read()
        //         .as_ref()
        //         .unwrap()
        //         .get_coordinates()
        //         .await
        //         .unwrap()
        // });
        let latest_coords = use_geolocation();

        let latest_coords = match latest_coords() {
            Ok(v) => v,
            Err(e) => {
                let e = format!("Initializing: {:?}", e);
                return rsx!(p { "{e}" });
            }
        };

        // Google maps embed api key
        //let key = std::env::var("DIOXUS_GEOLOCATION_MAP_KEY").unwrap();

        rsx!(
            div {
                style: "text-align: center;",
                h1 { "Warwick POIs - Collect 'em all!" }

                List{latitude: latest_coords.latitude, longitude: latest_coords.longitude}

                // Google maps embed
                //iframe {
                //    width: "400",
                //    height: "400",
                //    style: "border: 1px solid black",
                //    src: "https://www.google.com/maps/embed/v1/view?key={key}&center={latest_coords.latitude},{latest_coords.longitude}&zoom=16",
                //}
            }
        )
    }
    #[cfg(not(target_family = "wasm"))]
    rsx! {}
}

#[component]
fn List(latitude: f64, longitude: f64) -> Element {
    let mut dists: Vec<(usize, f64)> = POIS
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
    rsx! {
        h2 { "Your closest POIs are:" }
        for poi in &dists {
            p {"POI {POIS[poi.0].name} at {(poi.1 * 10000.).floor()}m"}
        }

        h2 {"Your current position is:"}
        p { "Latitude: {latitude} | Longitude: {longitude}" }
    }
}
