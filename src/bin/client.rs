use blokus_model::blokus_model_client::BlokusModelClient;
use blokus_model::State as ModelInput;


pub mod blokus_model {
    tonic::include_proto!("blokusmodel");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = BlokusModelClient::connect("http://[::1]:50051").await?;

    let request = tonic::Request::new(ModelInput {
        data: vec![false; 400 * 28],
    });

    let response = client.predict(request).await?;

    println!("RESPONSE={:?}", response);

    Ok(())
}