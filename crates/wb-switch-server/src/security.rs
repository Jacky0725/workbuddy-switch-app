//! Loopback browser sessions. Every API, including legacy routes, passes this layer.
use axum::{
    body::Body,
    extract::State,
    http::{header, Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
    Json, Router,
};
use std::sync::Arc;

pub struct LocalSession {
    pub port: u16,
    secret: String,
}
impl LocalSession {
    pub fn new(port: u16) -> Arc<Self> {
        Arc::new(Self {
            port,
            secret: format!(
                "{}{}",
                uuid::Uuid::new_v4().simple(),
                uuid::Uuid::new_v4().simple()
            ),
        })
    }
    fn trusted(&self, request: &Request<Body>) -> bool {
        let host = request
            .headers()
            .get(header::HOST)
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");
        if host != format!("127.0.0.1:{}", self.port) && host != format!("localhost:{}", self.port)
        {
            return false;
        }
        if let Some(origin) = request.headers().get(header::ORIGIN) {
            if origin.to_str().ok() != Some(format!("http://{host}").as_str()) {
                return false;
            }
        }
        if let Some(site) = request.headers().get("sec-fetch-site") {
            if !matches!(site.to_str().unwrap_or(""), "same-origin" | "none") {
                return false;
            }
        }
        true
    }
    fn authenticated(&self, request: &Request<Body>) -> bool {
        let Some(cookie) = request
            .headers()
            .get(header::COOKIE)
            .and_then(|v| v.to_str().ok())
        else {
            return false;
        };
        let Some(value) = cookie
            .split(';')
            .find_map(|part| part.trim().strip_prefix("wbm_session="))
        else {
            return false;
        };
        if value.len() != self.secret.len() {
            return false;
        }
        value
            .bytes()
            .zip(self.secret.bytes())
            .fold(0u8, |acc, (a, b)| acc | (a ^ b))
            == 0
    }
}
pub async fn protect(
    State(session): State<Arc<LocalSession>>,
    request: Request<Body>,
    next: Next,
) -> Response {
    if !session.trusted(&request) {
        return (
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({"error":"请求来源不受信任"})),
        )
            .into_response();
    }
    if request.uri().path() == "/api/session" && request.method() == "GET" {
        let mut response = Json(serde_json::json!({"authenticated":true})).into_response();
        response.headers_mut().insert(
            header::SET_COOKIE,
            format!(
                "wbm_session={}; Path=/; HttpOnly; SameSite=Strict",
                session.secret
            )
            .parse()
            .unwrap(),
        );
        response
            .headers_mut()
            .insert(header::CACHE_CONTROL, "no-store".parse().unwrap());
        return response;
    }
    if request.uri().path().starts_with("/api/") && !session.authenticated(&request) {
        return (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({"error":"本机会话已失效，请刷新页面"})),
        )
            .into_response();
    }
    let mut response = next.run(request).await;
    response
        .headers_mut()
        .insert("x-content-type-options", "nosniff".parse().unwrap());
    response
        .headers_mut()
        .insert("referrer-policy", "no-referrer".parse().unwrap());
    response.headers_mut().insert("content-security-policy", "default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'; img-src 'self' data:; font-src 'self'; connect-src 'self'; object-src 'none'; base-uri 'none'; frame-ancestors 'none'".parse().unwrap());
    response
        .headers_mut()
        .insert(header::CACHE_CONTROL, "no-store".parse().unwrap());
    response
}
pub fn wrap(router: Router, port: u16) -> Router {
    router.layer(axum::middleware::from_fn_with_state(
        LocalSession::new(port),
        protect,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tower::ServiceExt;
    fn router() -> Router {
        wrap(
            Router::new().route("/api/private", axum::routing::get(|| async { "private" })),
            57890,
        )
    }
    async fn request(
        router: Router,
        path: &str,
        host: &str,
        origin: Option<&str>,
        cookie: Option<&str>,
    ) -> Response {
        let mut req = Request::builder().uri(path).header(header::HOST, host);
        if let Some(value) = origin {
            req = req.header(header::ORIGIN, value);
        }
        if let Some(value) = cookie {
            req = req.header(header::COOKIE, value);
        }
        router
            .oneshot(req.body(Body::empty()).unwrap())
            .await
            .unwrap()
    }
    #[tokio::test]
    async fn all_api_routes_require_session_and_reject_foreign_sources() {
        let app = router();
        assert_eq!(
            request(app.clone(), "/api/private", "127.0.0.1:57890", None, None)
                .await
                .status(),
            StatusCode::UNAUTHORIZED
        );
        let response = request(app.clone(), "/api/session", "127.0.0.1:57890", None, None).await;
        assert_eq!(response.status(), StatusCode::OK);
        let cookie = response.headers()[header::SET_COOKIE]
            .to_str()
            .unwrap()
            .split(';')
            .next()
            .unwrap();
        assert!(response.headers()[header::SET_COOKIE]
            .to_str()
            .unwrap()
            .contains("HttpOnly; SameSite=Strict"));
        assert_eq!(
            request(
                app.clone(),
                "/api/private",
                "127.0.0.1:57890",
                Some("http://127.0.0.1:57890"),
                Some(cookie)
            )
            .await
            .status(),
            StatusCode::OK
        );
        for (host, origin) in [
            ("foreign.invalid:57890", None),
            ("127.0.0.1:57891", None),
            ("127.0.0.1:57890", Some("https://foreign.invalid")),
            ("127.0.0.1:57890", Some("null")),
        ] {
            assert_eq!(
                request(app.clone(), "/api/session", host, origin, None)
                    .await
                    .status(),
                StatusCode::FORBIDDEN
            );
        }
        assert_eq!(
            request(
                app,
                "/api/private",
                "127.0.0.1:57890",
                None,
                Some("wbm_session=wrong")
            )
            .await
            .status(),
            StatusCode::UNAUTHORIZED
        );
    }
    #[tokio::test]
    async fn session_cannot_be_reused_after_server_restart() {
        let response = request(router(), "/api/session", "localhost:57890", None, None).await;
        let cookie = response.headers()[header::SET_COOKIE]
            .to_str()
            .unwrap()
            .split(';')
            .next()
            .unwrap();
        assert_eq!(
            request(
                router(),
                "/api/private",
                "localhost:57890",
                None,
                Some(cookie)
            )
            .await
            .status(),
            StatusCode::UNAUTHORIZED
        );
    }
}
