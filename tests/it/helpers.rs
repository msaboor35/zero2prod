use actix_http::Request;
use actix_web::test;
use serde::Serialize;

pub fn get_confirmation_link(body: &[u8]) -> String {
    let body: serde_json::Value = serde_json::from_slice(body).unwrap();

    let get_link = |s: &str| {
        let links: Vec<_> = linkify::LinkFinder::new()
            .links(s)
            .filter(|l| *l.kind() == linkify::LinkKind::Url)
            .collect();

        assert_eq!(links.len(), 1);
        links[0].as_str().to_owned()
    };

    let html_link = get_link(body["Messages"][0]["HTMLPart"].as_str().unwrap());
    let text_link = get_link(body["Messages"][0]["TextPart"].as_str().unwrap());

    assert_eq!(html_link, text_link);
    html_link
}

pub fn post_subscription_request(form: impl Serialize) -> Request {
    test::TestRequest::post()
        .uri("/subscriptions")
        .set_form(form)
        .to_request()
}

pub fn post_newsletter(body: &serde_json::Value) -> Request {
    test::TestRequest::post()
        .uri("/newsletter")
        .set_json(body)
        .to_request()
}
