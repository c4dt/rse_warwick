use crate::Route;
use dioxus::prelude::*;

const POI_CSS: Asset = asset!("/assets/styling/poi.css");

/// The Blog page component that will be rendered when the current route is `[Route::Blog]`
///
/// The component takes a `id` prop of type `i32` from the route enum. Whenever the id changes, the component function will be
/// re-run and the rendered HTML will be updated.
#[component]
pub fn Poi(id: i32) -> Element {
    rsx! {
        document::Link { rel: "stylesheet", href: POI_CSS }

        div {
            id: "poi",

            // Content
            h1 { "This is poi #{id}!" }
            p { "In poi #{id}..." }

            // Navigation links
            // The `Link` component lets us link to other routes inside our app. It takes a `to` prop of type `Route` and
            // any number of child nodes.
            Link {
                // The `to` prop is the route that the link should navigate to. We can use the `Route` enum to link to the
                // blog page with the id of -1. Since we are using an enum instead of a string, all of the routes will be checked
                // at compile time to make sure they are valid.
                to: Route::Poi { id: id - 1 },
                "Previous"
            }
            span { " <---> " }
            Link {
                to: Route::Poi { id: id + 1 },
                "Next"
            }
        }
    }
}
