use actix_cors::Cors;
use actix_web::middleware::Logger;
use actix_web::{error, middleware, web, App, Error, HttpResponse, HttpServer};
use env_logger::Env;
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Mutex;

const MAX_SIZE: usize = 262_144; // max payload size is 256k

async fn health() -> Result<HttpResponse, Error> {
    let health = json!({"status": "ok"});
    Ok(HttpResponse::Ok().json(health))
}

async fn get_product(
    data: web::Data<AppState>,
    path: web::Path<ProductInfo>,
) -> Result<HttpResponse, Error> {
    let products = data.products.lock().unwrap();

    // find product by id in products
    let index = products
        .iter()
        .position(|p| p.id == path.product_id)
        .unwrap();

    Ok(HttpResponse::Ok().json(products[index].clone()))
}

async fn get_products(data: web::Data<AppState>) -> Result<HttpResponse, Error> {
    let products = data.products.lock().unwrap();
    Ok(HttpResponse::Ok().json(products.to_vec()))
}

async fn add_product(
    data: web::Data<AppState>,
    mut payload: web::Payload,
) -> Result<HttpResponse, Error> {
    let mut products = data.products.lock().unwrap();
    let new_id = products.len() as i32 + 2;

    // payload is a stream of Bytes objects
    let mut body = web::BytesMut::new();
    while let Some(chunk) = payload.next().await {
        let chunk = chunk?;
        // limit max size of in-memory payload
        if (body.len() + chunk.len()) > MAX_SIZE {
            return Err(error::ErrorBadRequest("overflow"));
        }
        body.extend_from_slice(&chunk);
    }

    // body is loaded, now we can deserialize serde-json
    let mut product = serde_json::from_slice::<Product>(&body)?;

    // update product id
    product.id = new_id;

    // add product to products
    products.push(product.clone());

    Ok(HttpResponse::Ok().json(product))
}

async fn update_product(
    data: web::Data<AppState>,
    mut payload: web::Payload,
) -> Result<HttpResponse, Error> {
    let mut products = data.products.lock().unwrap();

    // payload is a stream of Bytes objects
    let mut body = web::BytesMut::new();
    while let Some(chunk) = payload.next().await {
        let chunk = chunk?;
        // limit max size of in-memory payload
        if (body.len() + chunk.len()) > MAX_SIZE {
            return Err(error::ErrorBadRequest("overflow"));
        }
        body.extend_from_slice(&chunk);
    }

    // body is loaded, now we can deserialize serde-json
    let product = serde_json::from_slice::<Product>(&body)?;

    // replace product with same id
    let index = products.iter().position(|p| p.id == product.id).unwrap();
    products[index] = product.clone();

    Ok(HttpResponse::Ok().json(product))
}

async fn delete_product(
    data: web::Data<AppState>,
    path: web::Path<ProductInfo>,
) -> Result<HttpResponse, Error> {
    let mut products = data.products.lock().unwrap();

    // find product by id in products
    let index = products
        .iter()
        .position(|p| p.id == path.product_id)
        .unwrap();

    // remove product from products
    products.remove(index);

    Ok(HttpResponse::Ok().body(""))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let products = vec![
        Product {
            id: 1,
            name: "ZenoFit Tracker".to_string(),
            price: 99.99,
            description: "Monitor your heart rate, calories, steps, and more with this ultimate fitness tracker. It syncs with your smartphone and gives you personalized feedback and coaching. ZenoFit Tracker is your best companion for your fitness goals.".to_string(),
            image: "/placeholder.png".to_string()
        },
        Product {
            id: 2,
            name: "Airy Purifier Mini".to_string(),
            price: 79.99,
            description: "Clean the air in your home or office with this compact and powerful air purifier. It removes dust, pollen, smoke, odors, and bacteria. It also has a HEPA filter that lasts up to 6 months. Airy Purifier Mini is the device that breathes fresh air into your life.".to_string(),
            image: "/placeholder.png".to_string()
        },
        Product {
            id: 3,
            name: "Zenith Wireless Headphones".to_string(),
            price: 199.99,
            description: "Enjoy high-quality sound and noise cancellation with these premium headphones. They have a sleek design and a comfortable fit. They also have a long battery life and a built-in microphone. Zenith Wireless Headphones are the ultimate sound experience for your ears.".to_string(),
            image: "/placeholder.png".to_string()
        },
        Product {
            id: 4,
            name: "Hydrate Smart Bottle".to_string(),
            price: 39.99,
            description: "Track your water intake and remind yourself to drink more with this smart water bottle. It connects to your smartphone and shows you your hydration level and goals. It also glows in different colors to motivate you. Hydrate Smart Bottle is the bottle that keeps you hydrated and healthy.".to_string(),
            image: "/placeholder.png".to_string()
        },
        Product {
            id: 5,
            name: "Sleepy Weighted Mask".to_string(),
            price: 29.99,
            description: "Fall asleep faster and deeper with this sleep mask. It has a weighted design that applies gentle pressure to your eyes and temples. It also has a lavender scent that relaxes your mind and body. Sleepy Weighted Mask is the mask that helps you sleep better.".to_string(),
            image: "/placeholder.png".to_string()
        },
        Product {
            id: 6,
            name: "FlexiFit Yoga Mat".to_string(),
            price: 49.99,
            description: "Practice yoga with the perfect balance of comfort and stability with this yoga mat. It has a non-slip surface that grips the floor and prevents sliding. It also has a cushioned layer that supports your joints and spine. FlexiFit Yoga Mat is the mat that enhances your yoga experience.".to_string(),
            image: "/placeholder.png".to_string()
        },
        Product {
            id: 7,
            name: "Glow Ionic Dryer".to_string(),
            price: 89.99,
            description: "Dry your hair faster and smoother than ever before with this hair dryer. It uses ionic technology to reduce frizz and static electricity. It also has a ceramic coating that protects your hair from heat damage. Glow Ionic Dryer is the dryer that gives your hair a healthy glow.".to_string(),
            image: "/placeholder.png".to_string()
        },
        Product {
            id: 8,
            name: "Breathe Aroma Diffuser".to_string(),
            price: 49.99,
            description: "Fill your space with soothing scents and mood lighting with this aromatherapy diffuser. It comes with 6 different essential oils that have various benefits for your health and well-being. It also has a timer and a mist mode. Breathe Aroma Diffuser is the device that creates a relaxing atmosphere for you.".to_string(),
            image: "/placeholder.png".to_string()
        },
        Product {
            id: 9,
            name: "LumiSkin LED Device".to_string(),
            price: 149.99,
            description: "Rejuvenate your skin with this revolutionary device that uses LED light therapy. It reduces wrinkles, fine lines, dark spots, and acne. It also boosts collagen and elastin production. LumiSkin LED Device is the secret to a younger-looking skin.".to_string(),
            image: "/placeholder.png".to_string()
        },
        Product {
            id: 10,
            name: "FlexiFit Yoga Ball".to_string(),
            price: 39.99,
            description: "Improve your balance and flexibility with this high-quality yoga ball. It is made of anti-burst PVC material that can support up to 2200 lbs. It also comes with a pump and a workout guide. FlexiFit Yoga Ball is the perfect tool for your yoga practice and physical therapy.".to_string(),
            image: "/placeholder.png".to_string()
        }
    ];

    let product_state = web::Data::new(AppState {
        products: Mutex::new(products.to_vec()),
    });

    println!("Listening on http://0.0.0.0:3002");

    env_logger::init_from_env(Env::default().default_filter_or("info"));

    HttpServer::new(move || {
        let cors = Cors::permissive();

        App::new()
            .wrap(cors)
            .wrap(Logger::default())
            .wrap(Logger::new("%a %{User-Agent}i"))
            .wrap(middleware::DefaultHeaders::new().add(("X-Version", "0.2")))
            .app_data(product_state.clone())
            .route("/health", web::get().to(health))
            .route("/health", web::head().to(health))
            .route("/{product_id}", web::get().to(get_product))
            .route("/", web::get().to(get_products))
            .route("/", web::post().to(add_product))
            .route("/", web::put().to(update_product))
            .route("/{product_id}", web::delete().to(delete_product))
    })
    .bind(("0.0.0.0", 3002))?
    .run()
    .await
}

struct AppState {
    products: Mutex<Vec<Product>>,
}

#[derive(Serialize, Deserialize, Clone)]
struct Product {
    id: i32,
    name: String,
    price: f32,
    description: String,
    image: String,
}

#[derive(Deserialize)]
struct ProductInfo {
    product_id: i32,
}
