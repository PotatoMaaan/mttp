use super::{Handlers, RegisteredRoute};
use crate::http::HttpRequest;
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
        handler: RegisteredRoute<Arc<State>>,
    },
    Dynamic {
        route: PathBuf,
        dynamic_component_positions: HashMap<usize, OsString>,
        handler: RegisteredRoute<Arc<State>>,
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
    not_found_handler: RegisteredRoute<Arc<State>>,
    method_not_allowed_handler: RegisteredRoute<Arc<State>>,
    current_request: &HttpRequest,
) -> RegisteredRoute<Arc<State>> {
    if let Some(handler) = match_route(routes, current_request) {
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
) -> Option<RegisteredRoute<Arc<State>>> {
    let req_route = Path::new(&request.route);

    routes.iter().find_map(|route| match route {
        Route::Static { route, handler } => {
            if req_route == route {
                Some(handler.clone())
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
