pub async fn send<ResponseData, Vars>(
    url: &str,
    operation: cynic::Operation<ResponseData, Vars>,
) -> Result<ResponseData, Box<dyn std::error::Error>>
where
    Vars: cynic::serde::Serialize,
    ResponseData: cynic::serde::de::DeserializeOwned,
{
    let response = reqwest::Client::new()
        .post(url)
        .header("User-Agent", "obmarg/cynic")
        .json(&operation)
        .send()
        .await
        .unwrap();

    let response_data = response
        .json::<cynic::GraphQlResponse<ResponseData>>()
        .await?;

    if let Some(_errors) = response_data.errors {
        println!("{:?}", _errors);
        return Err("GraphQL server returned errors".into());
        // TODO: Better errors here
        //Err(errors)?
    }

    Ok(response_data.data.expect("response data"))
}
