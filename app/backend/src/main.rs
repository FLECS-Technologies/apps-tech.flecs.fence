use user_manager::router::build_router;

use tower_http::services::ServeDir;

#[tokio::main]
async fn main() {
    let router = build_router().fallback_service(ServeDir::new("./static"));

    let listener = tokio::net::TcpListener::bind("127.0.0.1:27000")
        .await
        .unwrap();

    axum::serve(listener, router).await.unwrap();

    // let client_registry = client_registry();
    // let authorizer = AuthMap::new(RandomGenerator::new(16));
    // let solicitor = FnSolicitor(|_req| Ok(())); // Simplified user consent
    // let mut oauth = Endpoint::new(client_registry, authorizer, solicitor);
    // match oauth.authorize(req.into_inner()).await {
    //     Ok(response) => Ok(response.into()),
    //     Err(err) => Err(err.into()),
    // }
}

// async fn token(
//     mut req: OAuthRequest<WebRequest>,
// ) -> Result<OAuthResponse<WebResponse>, OAuthFailure> {
//     let client_registry = client_registry();
//     let authorizer = AuthMap::new(RandomGenerator::new(16));
//     let issuer = TokenMap::new(RandomGenerator::new(16));
//     let mut oauth = Endpoint::new(client_registry, authorizer, issuer);
//     match oauth.token(req.into_inner()).await {
//         Ok(response) => Ok(response.into()),
//         Err(err) => Err(err.into()),
//     }
// }
//
// async fn authorize(
//     mut req: OAuthRequest<WebRequest>,
// ) -> Result<OAuthResponse<WebResponse>, OAuthFailure> {
// }
