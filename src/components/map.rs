use std::{collections::HashMap, fmt::Error};

use anyhow::Result;
use dioxus::{fullstack::once_cell::sync::OnceCell, logger::tracing, prelude::*};
use dioxus_leaflet::{Map, MapMarker, MapPosition, MarkerIcon};
use flarch::{nodeids::U256, tasks::now};
use flmacro::VersionedSerde;
use serde::{Deserialize, Serialize};

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
    #[cfg(feature = "web")]
    {
        use dioxus_sdk::geolocation::{init_geolocator, use_geolocation, PowerMode};
        use flarch::tasks::wait_ms;
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
                List{latitude: latest_coords.latitude, longitude: latest_coords.longitude}
            }
        )
    }
    #[cfg(not(target_family = "wasm"))]
    rsx! {}
}

#[component]
fn List(latitude: f64, longitude: f64) -> Element {
    #[cfg(feature = "web")]
    {
        use dioxus_sdk::storage::*;

        use crate::components::storage::store_user;

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
        let mut user_id = use_persistent("user_id", || U256::rnd());
        let mut user_name =
            use_persistent("user_name", || names::Generator::default().next().unwrap());
        use_resource(move || async move {
            store_user(user_id(), user_name()).await;
        });
        let pos = format!("{:.4}/{:.4}", latitude, longitude);
        rsx! {
            if distance > 20 {
                p { "{user_name}, your closest POI is {name} at {distance}m - get closer than 20m" }
                LocationTracker{poi: closest.0, latitude: latitude, longitude: longitude}
            } else {
                p { "{user_name}, you're at POI {name}!" }
                Messages{poi: closest.0}
            }
        }
    }
    #[cfg(not(feature = "web"))]
    rsx! {}
}

#[component]
fn Messages(poi: usize) -> Element {
    #[cfg(feature = "web")]
    {
        use flarch::tasks::wait_ms;

        use crate::components::storage::{add_message, get_messages};

        let mut messages = use_server_future(move || get_messages(poi))?;
        let mut input_text = use_signal(|| String::new());

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
                        add_message(U256::rnd(), poi, input_text()).await;
                        messages.restart();
                    }
                },
                { "Submit" }
            }

            if let Some(Ok(msgs)) = messages(){
                if msgs.len() > 0{
                    p{"Here are the messages for {_POIS[poi].name}"}
                    for msg in msgs.iter().rev() {
                        p{"-- {msg.message:?}"}
                    }
                } else {
                    p{"No messages found"}
                }
            }
        }
    }
    #[cfg(not(feature = "web"))]
    rsx! {}
}

#[component]
fn LocationTracker(poi: usize, latitude: f64, longitude: f64) -> Element {
    tracing::info!("New location {latitude} / {longitude}");
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
        Map {
            initial_position: MapPosition::new(latitude, longitude, 32.),
            markers: path_markers,
            height: "500px",
        }
    }
}
