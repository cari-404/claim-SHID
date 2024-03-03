/*
This Is a first version of get_vouchers_by_collections
This version using api reqwest
Whats new In 1.1.8 :
Add csrftoken function
restructure header
Whats new In 1.1.7 :
fix for windows 7 console
Whats new In 1.1.6 :
fix included ansicode on logs
Whats new In 1.1.5 :
Add function interactive_print
*/

use reqwest;
use reqwest::ClientBuilder;
use reqwest::Body;
use serde_json;
use serde_json::json;
use anyhow::Result;
use reqwest::Version;
use std::fs;
use std::fs::File;
use std::thread;
use std::time::Duration as StdDuration;
use std::io::{self, Read, Write};
use chrono::{Local, Duration, NaiveDateTime};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "Auto save voucher Shopee", about = "Make fast save from shopee.co.id")]
struct Opt {
    #[structopt(short, long, help = "time to run checkout")]
    time: Option<String>, 
}

fn extract_csrftoken(cookie_string: &str) -> String {
    let mut csrftoken = String::new();
    if let Some(token_index) = cookie_string.find("csrftoken=") {
        let token_start = token_index + "csrftoken=".len();
        if let Some(token_end) = cookie_string[token_start..].find(';') {
            csrftoken = cookie_string[token_start..token_start + token_end].to_string();
        }
    }
    csrftoken
}

async fn some_function(start: &str, end: &str, cookie_content: &str, selected_file: &str) -> Result<()> {

	// Mengonversi nama akun menjadi format folder yang sesuai
    let header_folder = format!("./header/{}/af-ac-enc-sz-token.txt", selected_file);
	
	// Membuat folder header jika belum ada
    fs::create_dir_all(&format!("./header/{}", selected_file))?;

    // Membuat file header jika belum ada
    if !File::open(&header_folder).is_ok() {
        let mut header_file = File::create(&header_folder)?;
        // Isi file header dengan konten default atau kosong sesuai kebutuhan
        header_file.write_all(b"ganti kode ini dengan sz-token valid")?;
    }

    // Baca isi file untuk header af-ac-enc-sz-token
    let mut sz_token_content = String::new();
    File::open(&header_folder)?.read_to_string(&mut sz_token_content)?;
	println!("sz-token:{}", sz_token_content);
    let cookie_content_owned = cookie_content.to_string();

    // Pass the cloned String to extract_csrftoken
    let csrftoken = extract_csrftoken(&cookie_content_owned);
    println!("csrftoken: {}", csrftoken);
	let csrftoken_string = csrftoken.to_string();
	let start: i64 = start.trim().parse().expect("Input tidak valid");

	let body_json = json!({
	  "voucher_promotionid": start as i64,
	  "signature": end.replace(char::is_whitespace, ""),
	  "security_device_fingerprint": sz_token_content,
	  "signature_source": "0"
	});
	
    let body_str = serde_json::to_string(&body_json)?;

    println!("{}", body_str);
	
	let mut headers = reqwest::header::HeaderMap::new();

	headers.insert("x-sap-access-f", reqwest::header::HeaderValue::from_static(""));
	headers.insert("x-shopee-client-timezone", reqwest::header::HeaderValue::from_static("Asia/Jakarta"));
	headers.insert("x-sap-access-t", reqwest::header::HeaderValue::from_static(""));
	headers.insert("af-ac-enc-dat", reqwest::header::HeaderValue::from_static(""));
	headers.insert("af-ac-enc-id", reqwest::header::HeaderValue::from_static(""));
	headers.insert("x-csrftoken", reqwest::header::HeaderValue::from_str(&csrftoken_string)?);
	headers.insert("user-agent", reqwest::header::HeaderValue::from_static("Android app Shopee appver=32010 app_type=1"));
	headers.insert("x-api-source", reqwest::header::HeaderValue::from_static("rn"));
	headers.insert("content-type", reqwest::header::HeaderValue::from_static("application/json"));
	headers.insert("accept", reqwest::header::HeaderValue::from_static("application/json"));
	headers.insert("if-none-match-", reqwest::header::HeaderValue::from_static("55b03-97d86fe6888b54a9c5bfa268cf3d922f"));
	headers.insert("shopee_http_dns_mode", reqwest::header::HeaderValue::from_static("1"));
	headers.insert("af-ac-enc-sz-token", reqwest::header::HeaderValue::from_str(&sz_token_content)?);
	headers.insert("origin", reqwest::header::HeaderValue::from_static("https://shopee.co.id"));
	headers.insert("referer", reqwest::header::HeaderValue::from_static("https://mall.shopee.co.id"));
	headers.insert("accept-encoding", reqwest::header::HeaderValue::from_static("gzip, deflate"));

	headers.insert(reqwest::header::COOKIE, reqwest::header::HeaderValue::from_str(&cookie_content)?);
	let body = Body::from(body_str);
	let client = ClientBuilder::new()
		.gzip(true)
		.use_rustls_tls() // Use Rustls for HTTPS
		.build()?;

	// Buat permintaan HTTP POST
	let response = client
		.post("https://mall.shopee.co.id/api/v2/voucher_wallet/save_voucher")
		.header("Content-Type", "application/json")
		.headers(headers)
		.body(body)
		.version(Version::HTTP_2) 
		.send()
		.await?;
	// Check for HTTP status code indicating an error
	//let http_version = response.version(); 		// disable output features
	//println!("HTTP Version: {:?}", http_version); // disable output features
	let text = response.text().await?;	
	println!("{}", text);
	
	Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Check if there are command line arguments
	
	println!("-------------------------------------------");
	println!("save_vouchers [Version 1.0.0]");
	println!("");
	println!("Dapatkan Info terbaru di https://google.com");
	println!("");
	println!("-------------------------------------------");
	let opt = Opt::from_args();
    let mut start = String::new();
    let mut end = String::new();
    
    // Display the list of available cookie files
    println!("Daftar file cookie yang tersedia:");
    let files = std::fs::read_dir("./akun")?;
    let mut file_options = Vec::new();
    for (index, file) in files.enumerate() {
        if let Ok(file) = file {
            let file_name = file.file_name();
            println!("{}. {}", index + 1, file_name.to_string_lossy());
            file_options.push(file_name.to_string_lossy().to_string());
        }
    }

    // Select the file number for the cookie
    let selected_file = loop {
        println!("Pilih nomor file cookie yang ingin digunakan:");
        let mut input = String::new();
        io::stdin().read_line(&mut input).expect("Gagal membaca baris");

        // Convert input to index number
        if let Ok(index) = input.trim().parse::<usize>() {
            if index > 0 && index <= file_options.len() {
                break file_options[index - 1].clone();
            }
        }
    };

    // Read the content of the selected cookie file
    let file_path = format!("./akun/{}", selected_file);
    let mut cookie_content = String::new();
    File::open(&file_path)?.read_to_string(&mut cookie_content)?;
	
	println!("Contoh input: Awal: 12905192072, Akhir: 12905192100");
    println!("Masukkan nilai start:");
    io::stdin().read_line(&mut start).expect("Gagal membaca baris");

    println!("Masukkan nilai akhir:");
    io::stdin().read_line(&mut end).expect("Gagal membaca baris");
	
	let task_time_str = opt.time.clone().unwrap_or_else(|| get_user_input("Enter task time (HH:MM:SS.NNNNNNNNN): "));
    let task_time_dt = parse_task_time(&task_time_str)?;

    // Process HTTP with common function
	countdown_to_task(&task_time_dt).await;
    some_function(&start, &end, &cookie_content, &selected_file).await?;
	
    Ok(())
}
async fn countdown_to_task(task_time_dt: &NaiveDateTime) {
    loop {
        let current_time = Local::now().naive_local();
        let task_time_naive = task_time_dt.time();
        let time_until_task = task_time_naive.signed_duration_since(current_time.time());

        if time_until_task < Duration::zero() {
            println!("\nTask completed! Current time: {}", current_time.format("%H:%M:%S.%3f"));
            tugas_utama();
            break;
        }

        let formatted_time = format_duration(time_until_task);
        print!("\r{}", formatted_time);
        io::stdout().flush().unwrap();

        thread::sleep(StdDuration::from_secs_f64(0.001));
    }
}

fn tugas_utama() {
    println!("Performing the task...");
    println!("\nTask completed! Current time: {}", Local::now().format("%H:%M:%S.%3f"));
}
fn get_user_input(prompt: &str) -> String {
    print!("{}", prompt);
    io::stdout().flush().unwrap();
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    input.trim().to_string()
}
fn format_duration(duration: Duration) -> String {
    let hours = duration.num_hours();
    let minutes = duration.num_minutes() % 60;
    let seconds = duration.num_seconds() % 60;
    let milliseconds = duration.num_milliseconds() % 1_000;

    format!("{:02}:{:02}:{:02}.{:03}", hours, minutes, seconds, milliseconds)
}
fn parse_task_time(task_time_str: &str) -> Result<NaiveDateTime> {
    match NaiveDateTime::parse_from_str(&format!("2023-01-01 {}", task_time_str), "%Y-%m-%d %H:%M:%S%.f") {
        Ok(dt) => Ok(dt),
        Err(e) => Err(e.into()),
    }
}
