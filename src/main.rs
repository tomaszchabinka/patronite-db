use patronite_db::{get_list_of_categories, get_list_of_creators};

fn main() {
    let categories = get_list_of_categories();
    for category in &categories {
        let creators = get_list_of_creators(category);

        for creator in &creators {
            println!("{:?}", creator);
        }
    }

    println!("Finished");
}
