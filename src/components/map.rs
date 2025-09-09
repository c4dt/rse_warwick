use std::{collections::HashMap, fmt::Error};

use anyhow::Result;
use dioxus::{fullstack::once_cell::sync::OnceCell, logger::tracing, prelude::*};
use dioxus_leaflet::{Map, MapMarker, MapPosition, MarkerIcon};
use flarch::{nodeids::U256, tasks::now};
use flmacro::VersionedSerde;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

struct _POI {
    latitude: f64,
    longitude: f64,
    name: &'static str,
}

const _POIS: [_POI; 11] = [
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
    _POI {
        latitude: 52.380092,
        longitude: -1.559804,
        name: "Forest 2011 - 2 Planet",
    },
    _POI {
        latitude: 52.380189,
        longitude: -1.560257,
        name: "Butterworth Bench",
    },
    _POI {
        latitude: 52.380320,
        longitude: -1.560126,
        name: "Forest 2011 - 3 Planet",
    },
    _POI {
        latitude: 52.380328,
        longitude: -1.559839,
        name: "Forest Planet - 3 2009",
    },
    _POI {
        latitude: 52.380010,
        longitude: -1.560788,
        name: "White Koan",
    },
    _POI {
        latitude: 52.377715,
        longitude: -1.567944,
        name: "The good, the bad",
    },
];

fn get_storage<T: DeserializeOwned + Serialize + std::fmt::Debug>(key: &str, default: T) -> T {
    let local_storage = web_sys::window().unwrap().local_storage().unwrap().unwrap();
    if let Some(s) = local_storage.get_item(key).unwrap() {
        if let Ok(value) = serde_json::from_str(&s) {
            return value;
        }
    }
    set_storage(key, &default);
    return default;
}

fn set_storage<T: Serialize + std::fmt::Debug>(key: &str, value: &T) {
    let local_storage = web_sys::window().unwrap().local_storage().unwrap().unwrap();
    local_storage.set_item(key, &serde_json::to_string(value).unwrap());
}

#[component]
pub fn MapPOI() -> Element {
    #[cfg(feature = "web")]
    rsx! {
        web::MapPOIWeb {  }
    }
    #[cfg(not(feature = "web"))]
    rsx! {}
}

#[cfg(feature = "web")]
mod web {
    use super::*;
    use crate::components::storage::{add_message, get_messages};
    use chrono::prelude::DateTime;
    use chrono::{Local, Utc};
    use dioxus_leaflet::MapOptions;
    use dioxus_sdk::geolocation::{
        core::Geocoordinates, init_geolocator, use_geolocation, PowerMode,
    };
    use flarch::tasks::{spawn_local, wait_ms};

    use crate::components::storage::store_user;

    #[component]
    pub fn MapPOIWeb() -> Element {
        let geolocator = init_geolocator(PowerMode::High);
        let latest_coords_caller = use_geolocation();
        let latest_coords = match latest_coords_caller() {
            Ok(v) => v,
            Err(e) => {
                return rsx! {p{"Error"}};
            }
        };

        rsx!(
            div {
                style: "text-align: center;",
                h1 { "Warwick POIs - Collect 'em all!" }
                List{longitude: latest_coords.longitude, latitude: latest_coords.latitude}
                p { "(c) 2025 by Linus  Gasser for EPFL/C4DT" }
                a { href: "https://github.com/c4dt/rse_warwick", "Github Repo" }
            }
        )
    }

    #[component]
    fn List(longitude: f64, latitude: f64) -> Element {
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
        let (name, distance) = (
            _POIS[closest.0].name,
            (closest.1 * 100000.).floor() as usize,
        );
        let user_id = get_storage("user_id", U256::rnd());
        let user_name = get_storage("user_name", names::Generator::default().next().unwrap());
        spawn_local(async move {
            store_user(user_id, get_storage("user_name", "Unknown".to_string())).await;
        });

        let mut update = use_signal(|| 0);
        use_resource(move || async move {
            loop {
                wait_ms(10).await;
                *update.write() += 1;
            }
        });

        rsx! {
            if distance < 20 {
                p { "{user_name}, you're at POI {name}!" }
                Messages{poi: closest.0}
            } else {
                p { "{user_name}, your closest POI is {name} at {distance}m - get closer than 20m" }
            }
            LocationTracker{poi: closest.0, latitude: latitude, longitude: longitude, update: update}
        }
    }

    fn unix_to_str(unix: i64) -> String {
        let datetime = DateTime::<Utc>::from_timestamp_millis(unix).unwrap();
        datetime.with_timezone(&Local).to_rfc2822()
    }

    #[component]
    fn Messages(poi: usize) -> Element {
        let mut messages = use_server_future(move || get_messages(poi))?;
        let mut input_text = use_signal(|| String::new());
        let mut user_id = get_storage("user_id", U256::rnd());

        rsx! {
            textarea {
                value: "{input_text}",
                oninput: move |e| input_text.set(e.value()),
                placeholder: "Enter multi-line text",
                rows: "4",
                cols: "50"
            }

            br{}

            button {
                onclick: move |_| {
                    async move {
                        add_message(user_id, poi, input_text()).await;
                        messages.restart();
                    }
                },
                { "Submit" }
            }

            if let Some(Ok(msgs)) = messages(){
                if msgs.len() > 0{
                    p{"Here are the messages for {_POIS[poi].name}"}
                    for msg in msgs.iter().rev() {
                        p{"-- '{msg.sender}' wrote ''{msg.message}'' at {unix_to_str(msg.time)}"}
                    }
                } else {
                    p{"No messages found"}
                }
            }
        }
    }

    #[component]
    fn LocationTracker(
        poi: usize,
        latitude: f64,
        longitude: f64,
        update: Signal<usize>,
    ) -> Element {
        let mut path_markers: Vec<MapMarker> = _POIS
            .iter()
            .enumerate()
            .map(|(i, p)| MapMarker {
                lat: p.latitude,
                lng: p.longitude,
                title: (i != poi)
                    .then(|| p.name.into())
                    .unwrap_or(format!("**{}**", p.name)),
                description: None,
                icon: None,
                popup_options: None,
                custom_data: None,
            })
            .collect();
        path_markers.push(MapMarker {
            lat: latitude,
            lng: longitude,
            title: "Position".into(),
            description: None,
            icon: Some(MarkerIcon {
                icon_url: "https://img.icons8.com/define-location".into(),
                icon_size: Some((32, 32)),
                icon_anchor: None,
                popup_anchor: None,
                shadow_url: None,
                shadow_size: None,
            }),
            popup_options: None,
            custom_data: None,
        });

        rsx! {
            if update() % 500 > 0{
                Map {
                    initial_position: MapPosition::new(latitude, longitude, 32.),
                    markers: path_markers,
                    height: "500px",
                }
            }
        }
    }
}
