pub fn send_query<'a, ResponseData: 'a, Root: cynic::QueryRoot>(
    url: &str,
    selection_set: cynic::SelectionSet<'a, ResponseData, Root>,
) -> Result<ResponseData, Box<dyn std::error::Error>> {
    let query = cynic::Operation::query(selection_set);

    let response = reqwest::blocking::Client::new()
        .post(url)
        .header("User-Agent", "obmarg/cynic")
        .json(&query)
        .send()
        .unwrap();

    let response_data = query.decode_response(response.json()?)?;
    if let Some(_errors) = response_data.errors {
        println!("{:?}", _errors);
        return Err("GraphQL server returned errors".into());
        // TODO: Better errors here
        //Err(errors)?
    }

    Ok(response_data.data.expect("response data"))
}

pub fn send_mutation<'a, ResponseData: 'a, Root: cynic::MutationRoot>(
    url: &str,
    selection_set: cynic::SelectionSet<'a, ResponseData, Root>,
) -> Result<ResponseData, Box<dyn std::error::Error>> {
    let query = cynic::Operation::mutation(selection_set);

    let response = reqwest::blocking::Client::new()
        .post(url)
        .header("User-Agent", "obmarg/cynic")
        .json(&query)
        .send()
        .unwrap();

    let response_data = query.decode_response(response.json()?)?;
    if let Some(_errors) = response_data.errors {
        println!("{:?}", _errors);
        return Err("GraphQL server returned errors".into());
        // TODO: Better errors here
        //Err(errors)?
    }

    Ok(response_data.data.expect("response data"))
}
