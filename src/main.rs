use patronite_db::get_list_of_categories;

fn main() {
    let categories = get_list_of_categories();

    for category in categories {
        println!("{} ({}): {:?}", category.name, category.id, category.url);
    }

    println!("Finished");
}
