use blokus_model::blokus_model_client::BlokusModelClient;
use blokus_model::StateRepresentation;


pub mod blokus_model {
    tonic::include_proto!("blokusmodel");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = BlokusModelClient::connect("http://[::1]:50051").await?;

    let request = tonic::Request::new(StateRepresentation {
        boards: vec![false; 400 * 5],
        player: 1,
    });

    let response = client.predict(request).await?;
    let prediction = response.into_inner();
    let policy = prediction.policy;
    let value = prediction.value;

    println!("POLICY={:?}", policy);
    println!("VALUE={:?}", value);

    Ok(())
}
