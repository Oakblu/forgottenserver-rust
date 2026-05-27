use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum HttpMethod {
    Get,
    Post,
    Put,
    Delete,
}

pub type HandlerFn = fn(&str) -> HttpResponse;

#[derive(Debug, Clone)]
pub struct HttpResponse {
    pub status_code: u16,
    pub body: String,
    pub content_type: String,
}

pub struct Router {
    routes: HashMap<(HttpMethod, String), HandlerFn>,
}

impl Router {
    pub fn new() -> Self {
        Self {
            routes: HashMap::new(),
        }
    }

    pub fn add_route(&mut self, method: HttpMethod, path: &str, handler: HandlerFn) {
        self.routes.insert((method, path.to_string()), handler);
    }

    pub fn dispatch(&self, method: &HttpMethod, path: &str, body: &str) -> HttpResponse {
        let key = (method.clone(), path.to_string());
        match self.routes.get(&key) {
            Some(handler) => handler(body),
            None => HttpResponse {
                status_code: 404,
                body: "Not Found".to_string(),
                content_type: "text/plain".to_string(),
            },
        }
    }
}

impl Default for Router {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hello_handler(_body: &str) -> HttpResponse {
        HttpResponse {
            status_code: 200,
            body: "hello".to_string(),
            content_type: "text/plain".to_string(),
        }
    }

    fn post_handler(_body: &str) -> HttpResponse {
        HttpResponse {
            status_code: 201,
            body: "created".to_string(),
            content_type: "application/json".to_string(),
        }
    }

    fn world_handler(_body: &str) -> HttpResponse {
        HttpResponse {
            status_code: 200,
            body: "world".to_string(),
            content_type: "text/plain".to_string(),
        }
    }

    #[test]
    fn dispatch_calls_correct_handler_for_registered_route() {
        let mut router = Router::new();
        router.add_route(HttpMethod::Get, "/hello", hello_handler);
        let resp = router.dispatch(&HttpMethod::Get, "/hello", "");
        assert_eq!(resp.status_code, 200);
        assert_eq!(resp.body, "hello");
    }

    #[test]
    fn dispatch_returns_404_for_unknown_path() {
        let router = Router::new();
        let resp = router.dispatch(&HttpMethod::Get, "/unknown", "");
        assert_eq!(resp.status_code, 404);
    }

    #[test]
    fn get_and_post_routes_are_distinct() {
        let mut router = Router::new();
        router.add_route(HttpMethod::Get, "/resource", hello_handler);
        router.add_route(HttpMethod::Post, "/resource", post_handler);

        let get_resp = router.dispatch(&HttpMethod::Get, "/resource", "");
        let post_resp = router.dispatch(&HttpMethod::Post, "/resource", "");

        assert_eq!(get_resp.status_code, 200);
        assert_eq!(get_resp.body, "hello");
        assert_eq!(post_resp.status_code, 201);
        assert_eq!(post_resp.body, "created");
    }

    #[test]
    fn multiple_routes_coexist() {
        let mut router = Router::new();
        router.add_route(HttpMethod::Get, "/hello", hello_handler);
        router.add_route(HttpMethod::Get, "/world", world_handler);

        let r1 = router.dispatch(&HttpMethod::Get, "/hello", "");
        let r2 = router.dispatch(&HttpMethod::Get, "/world", "");
        let r3 = router.dispatch(&HttpMethod::Get, "/other", "");

        assert_eq!(r1.body, "hello");
        assert_eq!(r2.body, "world");
        assert_eq!(r3.status_code, 404);
    }

    #[test]
    fn post_route_not_matched_by_get() {
        let mut router = Router::new();
        router.add_route(HttpMethod::Post, "/only-post", post_handler);
        let resp = router.dispatch(&HttpMethod::Get, "/only-post", "");
        assert_eq!(resp.status_code, 404);
    }

    #[test]
    fn default_router_has_no_routes_and_returns_404() {
        // Mirrors C++ router() default arm: unknown type/path returns the
        // error/not-found response. `Default::default()` must behave like
        // `Router::new()` — empty routing table that 404s on every dispatch.
        let router: Router = Default::default();
        let resp = router.dispatch(&HttpMethod::Get, "/anything", "body");
        assert_eq!(resp.status_code, 404);
        assert_eq!(resp.body, "Not Found");
        assert_eq!(resp.content_type, "text/plain");

        // Default + add_route works identically to Router::new() + add_route.
        let mut router2: Router = Default::default();
        router2.add_route(HttpMethod::Put, "/x", hello_handler);
        let r = router2.dispatch(&HttpMethod::Put, "/x", "");
        assert_eq!(r.status_code, 200);
        assert_eq!(r.body, "hello");
    }

    #[test]
    fn put_and_delete_method_variants_dispatch_distinctly() {
        // Exercise HttpMethod::Put and HttpMethod::Delete variants so the
        // enum's complete method-dispatch surface is covered (parallel to
        // C++ router() switching on the request `type` string).
        let mut router = Router::new();
        router.add_route(HttpMethod::Put, "/r", hello_handler);
        router.add_route(HttpMethod::Delete, "/r", post_handler);

        let put_resp = router.dispatch(&HttpMethod::Put, "/r", "");
        let del_resp = router.dispatch(&HttpMethod::Delete, "/r", "");

        assert_eq!(put_resp.status_code, 200);
        assert_eq!(put_resp.body, "hello");
        assert_eq!(del_resp.status_code, 201);
        assert_eq!(del_resp.body, "created");
    }
}
