use dotenv::dotenv;
use influxdb::{Client, InfluxDbWriteable, WriteQuery};
use patronite_db::{get_list_of_categories, get_list_of_creators};
use std::{collections::HashSet, env, error::Error};
use tokio::task;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv().ok();
    let url = "https://eu-central-1-1.aws.cloud2.influxdata.com";
    let db_name = "snapshot";

    let client = Client::new(url, db_name).with_token(env::var("INFLUXDB_TOKEN")?);

    let categories: Vec<patronite_db::Category> =
        task::spawn_blocking(get_list_of_categories).await.unwrap();

    let mut updated_creators: HashSet<String> = HashSet::new();

    for category in categories {
        let mut additional_creators = vec![];

        let creators = task::spawn_blocking(move || get_list_of_creators(&category))
            .await
            .unwrap();

        for creator in creators {
            if updated_creators.get(&creator.url).is_none() {
                updated_creators.insert(String::from(&creator.url));
                additional_creators.push(creator);
            }
        }

        client
            .query(
                additional_creators
                    .iter()
                    .map(|summary| summary.clone().into_query("creators"))
                    .collect::<Vec<WriteQuery>>(),
            )
            .await?;
    }

    println!("Finished");

    Ok(())
}
