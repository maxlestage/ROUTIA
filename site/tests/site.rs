//! Chaque page à travers le client de test de Paquets (sans port).

#[tokio::test]
async fn pages_render() {
    let client = routia_site::server::app().test().await;

    let res = client.get("/").await;
    assert_eq!(res.status(), 200);
    let html = res.text();
    assert!(html.contains(r#"<html lang="fr""#));
    assert!(html.contains("Une habitude"));
    assert!(html.contains(r#"data-island="hero""#));
    assert!(html.contains(r#"data-island="tower""#));
    assert!(html.contains(r#"popover"#));
    assert!(html.contains("rel=\"canonical\""));

    let res = client.get("/app/").await;
    assert_eq!(res.status(), 200);
    let html = res.text();
    assert!(html.contains(r#"data-island="tracker""#));
    assert!(html.contains("Nouvelle habitude"));

    let res = client.get("/introuvable").await;
    assert_eq!(res.status(), 404);
    assert!(res.text().contains("noindex"));

    let res = client.get("/sitemap.xml").await;
    assert!(res.text().contains("/app/</loc>"));
    assert!(client.get("/robots.txt").await.text().contains("Sitemap:"));
    assert_eq!(client.get("/site.css").await.status(), 200);
}
