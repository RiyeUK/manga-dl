pub mod api {
    use anyhow::Result;
    use reqwest::Client;
    use serde_json::json;

    mod constants {
        pub const ENDPOINT: &str = "https://graphql.anilist.co/";
        pub const MANGA_ID_QUERY: &str = "
query ($id: Int) {
  Media (id: $id, type: MANGA) {
    title {
      english
    }
  }
}
";
    }

    pub async fn get_anilist_name(id: u32) -> Result<String> {
        let client = Client::new();
        let json = json!({"query": constants::MANGA_ID_QUERY, "variables": {"id": id}});
        let resp = client
            .post(constants::ENDPOINT)
            .header("Content-Type", "application/json")
            .header("Accept", "application/json")
            .body(json.to_string())
            .send()
            .await
            .unwrap()
            .text()
            .await;

        let result: serde_json::Value = serde_json::from_str(&resp?)?;
        Ok(result["data"]["Media"]["title"]["english"].to_string())
    }
}
