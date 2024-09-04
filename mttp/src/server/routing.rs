use super::{Handler, Handlers};
use crate::{
    http::{HttpRequest, Method},
    server::default_handlers,
    url,
};
use std::{
    collections::HashMap,
    ffi::OsString,
    fmt::Debug,
    path::{Path, PathBuf},
    sync::Arc,
};

#[derive(Debug, Clone)]
pub enum Route<State> {
    Static {
        route: PathBuf,
        handler: Handler<Arc<State>>,
    },
    Dynamic {
        route: PathBuf,
        dynamic_component_positions: HashMap<usize, OsString>,
        handler: Handler<Arc<State>>,
    },
}

pub fn build_dynamic_routes<State>(handlers: Handlers<State>) -> Vec<Route<State>> {
    handlers
        .into_iter()
        .map(|(route, handler)| {
            let path = Path::new(&route);

            let positions = path
                .components()
                .enumerate()
                .filter_map(|(pos, component)| {
                    let string = component.as_os_str().to_string_lossy();
                    if string.starts_with(':') {
                        Some((pos, component.as_os_str().to_owned()))
                    } else {
                        None
                    }
                })
                .collect::<HashMap<_, _>>();

            if positions.is_empty() {
                Route::Static {
                    route: PathBuf::from(route),
                    handler,
                }
            } else {
                Route::Dynamic {
                    dynamic_component_positions: positions,
                    route: PathBuf::from(route),
                    handler,
                }
            }
        })
        .collect()
}

pub fn router<State: 'static + Send + Sync>(
    routes: &[Route<State>],
    not_found_handler: Option<Handler<Arc<State>>>,
    method_not_allowed_handler: Option<Handler<Arc<State>>>,
    current_request: &HttpRequest,
) -> Handler<Arc<State>> {
    let not_found_handler = not_found_handler.unwrap_or(Handler {
        handler: default_handlers::not_found::<State>,
        method: Method::Get,
        params: HashMap::new(),
    });

    let method_not_allowed_handler = method_not_allowed_handler.unwrap_or(Handler {
        handler: default_handlers::handler_method_not_allowed::<State>,
        method: Method::Get,
        params: HashMap::new(),
    });

    if let Some(handler) = match_route(&routes, current_request) {
        if current_request.method == handler.method {
            handler.clone()
        } else {
            method_not_allowed_handler.clone()
        }
    } else {
        not_found_handler.clone()
    }
}

fn match_route<State>(
    routes: &[Route<State>],
    request: &HttpRequest,
) -> Option<Handler<Arc<State>>> {
    let (url, queryparams) = url::queryparams::parse_query_params_and_urldecode(&request.route);
    let req_route = Path::new(url);

    routes.iter().find_map(|route| match route {
        Route::Static { route, handler } => {
            if req_route == route {
                let mut handler = handler.clone();
                handler.params.extend(queryparams.clone());

                Some(handler)
            } else {
                None
            }
        }
        Route::Dynamic {
            route,
            dynamic_component_positions: positions,
            handler,
        } => {
            let route_len = route.components().count();
            let request_len = req_route.components().count();

            let mut handler = handler.clone();

            if route_len != request_len {
                return None;
            }

            if req_route
                .components()
                .zip(route.components())
                .enumerate()
                .map(|(curr_pos, (request_component, route_component))| {
                    if let Some(dynamic_part) = positions.get(&curr_pos) {
                        handler.params.extend(queryparams.clone());
                        let key = dynamic_part.to_string_lossy();
                        let key = key
                            .strip_prefix(':')
                            .expect("route parameter must have a :");

                        handler.params.insert(
                            key.to_owned(),
                            request_component.as_os_str().to_string_lossy().to_string(),
                        );
                        true
                    } else {
                        request_component == route_component
                    }
                })
                .all(|matched| matched)
            {
                Some(handler)
            } else {
                None
            }
        }
    })
}
