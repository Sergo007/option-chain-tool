use option_chain_tool::opt;

mod option_ext;
#[derive(Debug, Clone)]
struct User {
    profile: Option<Profile>,
    age: Option<i32>,
}

#[derive(Debug, Clone)]
struct Profile {
    address: Option<Address>,
}

#[derive(Debug, Clone)]
struct Address {
    city: Option<String>,
    street: String,
    some_field: Result<String, String>,
}

impl Address {
    fn get_street(&self) -> &String {
        &self.street
    }
    fn get_city(&self) -> Option<&String> {
        self.city.as_ref()
    }
}

fn main() {
    tracing_subscriber::fmt()
        .with_thread_ids(true)
        .with_file(true)
        .with_line_number(true)
        .with_target(true)
        .init();

    let user = User {
        age: Some(30),
        profile: Some(Profile {
            address: Some(Address {
                city: Some("New York".to_string()),
                street: "5th Avenue".to_string(),
                some_field: Ok("Some value".to_string()),
            }),
        }),
    };

    let a = opt!(user.age?);

    let a: Option<&String> = opt!(user.profile?.address?.city?);
    let a = opt!(user.profile?.address?.street);
    let a = opt!(user.profile?.address?.get_city()?);
    let a = opt!(user.profile?.address?.some_field?Err);
    let a = opt!(user.profile?.address?.some_field?Ok);

    println!("City: {:?}", a);
}
