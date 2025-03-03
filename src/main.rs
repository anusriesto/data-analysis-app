use std::{env, vec};
use std::error::Error;
use std::fs::File;
use std::io::{self,Write};
use axum::extract::State;
use axum::http::StatusCode;
use axum::routing::options_service;
use axum::serve::Listener;
use axum::{
    extract::{Multipart},
    routing::{post},
    Json, Router,
};
use std::io::Cursor;

use tower_http::cors::{CorsLayer, Any};
use serde::{Deserialize,Serialize};
use tokio::net::TcpListener;
use tokio::sync::Mutex;
use std::sync::Arc;
use calamine::{open_workbook_auto};
use axum_macros::debug_handler;

type SharedState = Arc<Mutex<String>>;



use csv::Reader;
use llm_chain::{executor,parameters,prompt,step::Step};

// #[tokio::main] //without API
// async fn main()->Result<(),Box<dyn Error>>{
//     let exec=executor!()?;
//     let file=File::open("data.csv")?;
//     let mut reader=Reader::from_reader(file);

//     let mut csv_data=String::new();
//     for result in reader.records(){
//         let record=result?;
//         csv_data.push_str(&record.iter().collect::<Vec<_>>().join(","));
//         csv_data.push('\n');
//     }


//     loop{
//         println!("Enter your prompt(or 'quit' to exit):");
//         io::stdout().flush()?;

//         let mut user_prompt=String::new();
//         io::stdin().read_line(&mut user_prompt)?;
//         user_prompt=user_prompt.trim().to_string();
        
//         if user_prompt.to_lowercase()=="quit"{
//             break;
//         }

//         let prompt_string=format!(
//             "you are a data analyst tasked with analysing a csv file. Understand the client's requirement then answer based on the data present in the csv file.
//             Question:{}\n\n CSV DATA:\n{}",
//             user_prompt,csv_data
//         );
//         let step=Step::for_prompt_template(prompt!("{}",&prompt_string));
//         let res=step.run(&parameters!(), &exec).await?;
//         println!("{}",res.to_immediate().await?.as_content());
//     }

    
//     Ok(())
// }

#[tokio::main]
async fn main()->Result<(),Box<dyn Error>>{
    let cors = CorsLayer::new()
        .allow_origin(Any) // Allow requests from any domain (change this for production)
        .allow_methods(Any)
        .allow_headers(Any);
    let csv_data=Arc::new(Mutex::new(String::new()));
    let shared_state = Arc::new(Mutex::new(String::new()));
    let app=Router::new()
    .route("/upload", post(upload_file))
    .route("/ask", post(handle_request))
    .with_state(csv_data)
    .layer(cors);

    

    //let listener=TcpListener::bind("127.0.0.1/8000").await?;
    let listener = tokio::net::TcpListener::bind("127.0.0.1:8000").await.unwrap();
    println!("🚀 Server running at successfully");
    axum::serve(listener, app).await.unwrap();
    Ok(())
}

async fn upload_file(State(state):State<SharedState>,mut multipart:Multipart,)->Result<Json<serde_json::Value>,axum::http::StatusCode>{
    while let Some(field)=multipart.next_field().await.unwrap(){
        let filename=field.file_name().unwrap_or("unknown").to_string();
        let data=field.bytes().await.unwrap();
        

        let parsed_data=if filename.ends_with(".csv"){
            parse_csv(&data).await
        // } else if filename.ends_with(".xlsx"){
        //     parse_excel(&data).await
        }else {
            return Err(axum::http::StatusCode::BAD_REQUEST);
        };

        let mut shared_csv=state.lock().await;
        *shared_csv=parsed_data;

    }
    Ok(Json(serde_json::json!({"status":"File updated successfully"})))
}
//parse csv file
async fn parse_csv(data:&[u8])->String{
    let mut reader=Reader::from_reader(data);
    let mut csv_data=String::new();
    for result in reader.records(){
        let record=result.unwrap();
        csv_data.push_str(&record.iter().collect::<Vec<_>>().join(","));
        csv_data.push('\n');
    }
    csv_data
}

// async fn parse_excel(data:&[u8])->String{
//     let mut workbook=open_workbook_auto(Cursor::new(data)).unwrap();
//     let mut excel_data=String::new();
//     if let Ok(range)=workbook.worksheet_range("sheet1"){
//         for row in range.rows(){
//             let row_data:Vec<String>=row.iter().map(|c| c.to_string()).collect();
//             excel_data.push_str(&row_data.join(","));
//             excel_data.push('\n');
//         }
//     }
//     excel_data

// }

//query Ai
#[derive(Deserialize,Serialize)]
struct UserRequest{
    prompt:String,
}

#[derive(Deserialize,Serialize)]
struct AiResponse{
    response:String,
}
#[debug_handler]
async fn handle_request(
    State(state): State<SharedState>,
    Json(payload): Json<UserRequest>,
) -> Result<Json<AiResponse>, StatusCode> {
    println!("📥 Received request: {:?}", payload.prompt); // ✅ Log request
    
    let csv_data = state.lock().await.clone();
    
    if csv_data.is_empty() {
        println!("⚠️ No data uploaded yet.");
        return Ok(Json(AiResponse { response: "No data uploaded yet".to_string() }));
    }

    let prompt_string = format!(
        "You are a data analyst, Your name is Anuj. Based on the presented data, answer the questions:\n\
        Question: {}\n\nDATA:\n{}",
        payload.prompt, csv_data
    );

    println!("🧠 AI Processing Prompt..."); // ✅ Log before execution

    let exec = executor!().map_err(|e| {
        println!("❌ Executor Error: {:?}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;  

    let step = Step::for_prompt_template(prompt!("{}",&prompt_string));

    let res = step.run(&parameters!(), &exec)
        .await
        .map_err(|e| {
            println!("❌ AI Execution Error: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let response_text = res.to_immediate()
        .await
        .map_err(|e| {
            println!("❌ Response Extraction Error: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .as_content()
        .to_string();
    
    println!("✅ AI Response: {}", response_text); // ✅ Log final response
    
    Ok(Json(AiResponse { response: response_text }))
}



