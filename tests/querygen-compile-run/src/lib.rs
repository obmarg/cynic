pub fn send<'a, ResponseData: 'a>(
    url: &str,
    operation: cynic::Operation<'a, ResponseData>,
) -> Result<ResponseData, Box<dyn std::error::Error>> {
    let response = reqwest::blocking::Client::new()
        .post(url)
        .header("User-Agent", "obmarg/cynic")
        .json(&operation)
        .send()
        .unwrap();

    let response_data = operation.decode_response(response.json()?)?;
    if let Some(_errors) = response_data.errors {
        println!("{:?}", _errors);
        return Err("GraphQL server returned errors".into());
        // TODO: Better errors here
        //Err(errors)?
    }

    Ok(response_data.data.expect("response data"))
}
