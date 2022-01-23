use serde_json::json;

pub fn send<ResponseData, Vars>(
    url: &str,
    operation: cynic::Operation<ResponseData, Vars>,
) -> Result<ResponseData, Box<dyn std::error::Error>> {
    let response = ureq::post(url)
        .set("User-Agent", "obmarg/cynic")
        .send_json(json!(operation))
        .unwrap();

    let response_data = operation.decode_response(response.into_json()?)?;
    if let Some(_errors) = response_data.errors {
        println!("{:?}", _errors);
        return Err("GraphQL server returned errors".into());
        // TODO: Better errors here
        //Err(errors)?
    }

    Ok(response_data.data.expect("response data"))
}
