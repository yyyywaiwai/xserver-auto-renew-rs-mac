mod account;
mod client;
mod db;
mod form;

use actix_web::{App, HttpServer, Responder, get, web};

#[get("/hello/{name}")]
async fn greet(name: web::Path<String>) -> impl Responder {
    format!("Hello {name}!")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| App::new().service(greet))
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}

// #[tokio::main]
// async fn main() {
//     // let url = "https://secure.xserver.ne.jp/xapanel/login/xvps/";

//     // let html = client
//     //     .get(url)
//     //     .send()
//     //     .await
//     //     .expect("Failed to send request")
//     //     .text()
//     //     .await
//     //     .expect("Failed to read response text");

//     // std::fs::write("data/response.html", html).expect("Failed to write response to file");

//     // let html =
//     //     std::fs::read_to_string("data/response.html").expect("Failed to read response from file");
//     // let base_url = Url::parse("https://secure.xserver.ne.jp/xapanel/login/xvps/").ok();
//     // let forms = extract_forms(&html, base_url.as_ref());
//     // let mut forms = forms
//     //     .into_iter()
//     //     .filter(|f| {
//     //         f.action
//     //             .as_ref()
//     //             .map(|a| a.contains("login"))
//     //             .unwrap_or(false)
//     //             && f.method.as_deref() == Some("POST")
//     //     })
//     //     .collect::<Vec<_>>();

//     // if forms.len() != 1 {
//     //     eprintln!("Expected exactly one login form, found: {}", forms.len());
//     //     std::process::exit(1);
//     // }

//     // let mut form = forms.pop().unwrap();

//     // for field in &mut form.fields {
//     //     match classify_field(field) {
//     //         FieldType::Id => {
//     //             field.value = Some("email".into());
//     //         }
//     //         FieldType::Password => {
//     //             field.value = Some("password".into());
//     //         }
//     //         _ => {}
//     //     }
//     // }

//     // println!("Form action: {:?}", form.action);
//     // println!("Form method: {:?}", form.method);
//     // for field in &form.fields {
//     //     println!(
//     //         "Field: name={}, type={}, value={:?}",
//     //         field.name, field.r#type, field.value
//     //     );
//     // }

//     // let params: HashMap<String, String> = HashMap::from_iter(
//     //     form.fields
//     //         .iter()
//     //         .filter_map(|f| f.value.as_ref().map(|v| (f.name.clone(), v.clone()))),
//     // );

//     // let res = client
//     //     .post(form.action.as_ref().unwrap())
//     //     .form(&params)
//     //     .send()
//     //     .await
//     //     .expect("Failed to send login request");

//     // println!("Login response: {:?}", res);

//     // let text = res.text().await.unwrap();
//     // std::fs::write("data/login_response.html", &text)
//     //     .expect("Failed to write login response to file");

//     // let text = std::fs::read_to_string("data/login_response.html")
//     //     .expect("Failed to read login response from file");

//     // let forms = extract_forms(&text, base_url.as_ref())
//     //     .pop()
//     //     .expect("No forms found in response");
//     // let mut params: HashMap<String, String> = HashMap::from_iter(
//     //     forms
//     //         .fields
//     //         .iter()
//     //         .filter_map(|f| f.value.as_ref().map(|v| (f.name.clone(), v.clone()))),
//     // );
//     // params.get_mut("auth_type").map(|v| *v = "auth_mail".into());

//     // let text = client
//     //     .post(forms.action.as_ref().unwrap())
//     //     .form(&params)
//     //     .send()
//     //     .await
//     //     .expect("Failed to send auth request")
//     //     .text()
//     //     .await
//     //     .expect("Failed to read auth response text");

//     // std::fs::write("data/auth_response.html", &text)
//     //     .expect("Failed to write auth response to file");

//     // let text = std::fs::read_to_string("data/auth_response.html")
//     //     .expect("Failed to read auth response from file");

//     // let url = "https://secure.xserver.ne.jp/xapanel/myaccount/loginauth/smssend"
//     //     .parse()
//     //     .unwrap();
//     // let forms = extract_forms(&text, Some(&url))
//     //     .into_iter()
//     //     .filter(|f| {
//     //         f.action
//     //             .as_ref()
//     //             .map(|a| a.contains("/do"))
//     //             .unwrap_or(false)
//     //     })
//     //     .collect::<Vec<_>>()
//     //     .pop()
//     //     .unwrap();

//     // let mut params: HashMap<String, String> = HashMap::from_iter(
//     //     forms
//     //         .fields
//     //         .iter()
//     //         .filter_map(|f| f.value.as_ref().map(|v| (f.name.clone(), v.clone()))),
//     // );
//     // params.get_mut("auth_code").map(|v| *v = "19273".into());

//     // let res = client
//     //     .post(forms.action.as_ref().unwrap())
//     //     .form(&params)
//     //     .send()
//     //     .await
//     //     .expect("Failed to send auth code request");

//     // println!("Auth code response: {:?}", res);
// }
