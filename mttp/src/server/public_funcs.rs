use super::{
    default_handlers::{self, make_default},
    HandlerFunc, MiddlewareFunc, RegisteredRoute, Server,
};
use crate::http::{HttpResponse, Method};
use std::{
    collections::HashMap,
    sync::{atomic::AtomicI64, Arc},
};

macro_rules! impl_method_func {
    ($name:ident, $method:ident) => {
        pub fn $name(
            &mut self,
            route: &str,
            handler: HandlerFunc<Arc<State>>,
            middleware: Vec<MiddlewareFunc<Arc<State>>>,
        ) {
            self.handlers.insert(
                route.to_owned(),
                RegisteredRoute {
                    handler,
                    method: Method::$method,
                    params: HashMap::new(),
                    specific_middlewares: middleware,
                },
            );
        }
    };
}

macro_rules! impl_specific_handler_func {
    ($name:ident) => {
        pub fn $name(&mut self, handler: HandlerFunc<Arc<State>>) {
            self.$name = RegisteredRoute {
                handler,
                method: Method::Get,
                params: HashMap::new(),
                specific_middlewares: Vec::new(),
            };
        }
    };
}

impl<State: 'static + Send + Sync> Server<State> {
    pub fn new(state: State) -> Self {
        Self {
            handlers: HashMap::new(),
            state: Arc::new(state),
            not_found_handler: make_default(default_handlers::not_found),
            method_not_allowd_handler: make_default(default_handlers::method_not_allowed),
            thread_counter: Arc::new(AtomicI64::new(0)),
            middlewares: Vec::new(),
            inspector: |_| {},
            error_handler: default_handlers::error,
        }
    }

    pub fn error_handler(&mut self, handler: fn(Box<dyn std::error::Error>) -> HttpResponse) {
        self.error_handler = handler
    }

    impl_method_func!(post, Post);
    impl_method_func!(get, Get);
    impl_method_func!(put, Put);
    impl_method_func!(patch, Patch);
    impl_method_func!(delete, Delete);

    impl_specific_handler_func!(not_found_handler);
    impl_specific_handler_func!(method_not_allowd_handler);

    pub fn middleware(&mut self, handler: MiddlewareFunc<Arc<State>>) {
        self.middlewares.push(handler);
    }

    pub fn inspector(&mut self, inspector: fn(&HttpResponse)) {
        self.inspector = inspector;
    }
}
