/*
This Is a first version of save_voucher
This version using api reqwest
Whats new In 1.2.1 :
fix failing result from collection_id (headers)
Whats new In 1.2.0 :
Universal input
Adjust time
Simple output
Whats new In 1.1.0 :
add modes
*/

use reqwest::{self, ClientBuilder, Version};
use serde_json;
use serde::Serialize;
use anyhow::Result;
use std::fs::File;
use std::io::{self, Read, Write};
use chrono::{Local, Duration, NaiveDateTime};
use structopt::StructOpt;

mod prepare;

#[derive(Debug, StructOpt)]
#[structopt(name = "Auto save voucher Shopee", about = "Make fast save from shopee.co.id")]
struct Opt {
	#[structopt(short, long, help = "time to run checkout")]
	time: Option<String>, 
}

#[derive(Serialize)]
struct SaveVoucherRequest {
	voucher_promotionid: i64,
	signature: String,
	security_device_fingerprint: String,
	signature_source: String,
}

enum Mode {
	Normal,
	Collection,
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

async fn some_function(start: &str, end: &str, cookie_content: &str) -> Result<()> {
	let cookie_content_owned = cookie_content.to_string();

	// Pass the cloned String to extract_csrftoken
	let csrftoken = extract_csrftoken(&cookie_content_owned);
	//println!("csrftoken: {}", csrftoken);
	let csrftoken_string = csrftoken.to_string();
	let start: i64 = start.trim().parse().expect("Input tidak valid");

	let body_json = SaveVoucherRequest {
	  voucher_promotionid: start as i64,
	  signature: end.to_string(),
	  security_device_fingerprint: String::new(),
	  signature_source: 0.to_string(),
	};
	
	let body_str = serde_json::to_string(&body_json)?;

	println!("{}", body_str);
	
	let mut headers = reqwest::header::HeaderMap::new();
	headers.insert("User-Agent", reqwest::header::HeaderValue::from_static("Android app Shopee appver=29330 app_type=1"));
	headers.insert("accept", reqwest::header::HeaderValue::from_static("application/json"));
	headers.insert("Content-Type", reqwest::header::HeaderValue::from_static("application/json"));
	headers.insert("x-api-source", reqwest::header::HeaderValue::from_static("rn"));
	headers.insert("if-none-match-", reqwest::header::HeaderValue::from_static("55b03-97d86fe6888b54a9c5bfa268cf3d922f"));
	headers.insert("shopee_http_dns_mode", reqwest::header::HeaderValue::from_static("1"));
	headers.insert("x-shopee-client-timezone", reqwest::header::HeaderValue::from_static("Asia/Jakarta"));
	headers.insert("af-ac-enc-dat", reqwest::header::HeaderValue::from_static(""));
	headers.insert("af-ac-enc-id", reqwest::header::HeaderValue::from_static(""));
	headers.insert("x-sap-access-t", reqwest::header::HeaderValue::from_static(""));
	headers.insert("x-sap-access-f", reqwest::header::HeaderValue::from_static(""));
	headers.insert("referer", reqwest::header::HeaderValue::from_static("https://mall.shopee.co.id/"));
	headers.insert("x-csrftoken", reqwest::header::HeaderValue::from_str(&csrftoken_string)?);
	headers.insert("af-ac-enc-sz-token", reqwest::header::HeaderValue::from_static(""));
	headers.insert(reqwest::header::COOKIE, reqwest::header::HeaderValue::from_str(&cookie_content)?);

	//println!("");
	//println!("header:{:#?}", headers);
	let mut attempt_count = 0;
	let max_attempts = 3; // Ubah angka sesuai kebutuhan Anda
	loop {
		let client = ClientBuilder::new()
			.gzip(true)
			.use_rustls_tls() // Use Rustls for HTTPS
			.build()?;

		// Buat permintaan HTTP POST
		let response = client
			.post("https://mall.shopee.co.id/api/v2/voucher_wallet/save_voucher")
			.header("Content-Type", "application/json")
			.headers(headers.clone())
			.body(body_str.clone())
			.version(Version::HTTP_2) 
			.send()
			.await?;
		// Check for HTTP status code indicating an error
		//let http_version = response.version(); 		// disable output features
		//println!("HTTP Version: {:?}", http_version); // disable output features
		let status = response.status();
		println!("{}", status);
		let text = response.text().await?;	
		if status == reqwest::StatusCode::OK {
			println!("{}", text);
			println!("Claim Berhasil!");
			break;
		} else if status == reqwest::StatusCode::IM_A_TEAPOT {
			println!("Gagal, status code: 418 - I'm a teapot. Mencoba kembali...");
			println!("{}", text);
			attempt_count += 1;
			if attempt_count >= max_attempts {
				println!("Batas percobaan maksimum tercapai.");
				break;
			}
			continue;
		}else {
			println!("Status: {}", status);
			break;
		}
	}
	Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
	// Check if there are command line arguments
	
	println!("-------------------------------------------");
	println!("save_vouchers [Version 1.2.1]");
	println!("");
	println!("Dapatkan Info terbaru di https://google.com");
	println!("");
	println!("-------------------------------------------");
	let opt = Opt::from_args();
	let mode = select_mode();

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
		let input = get_user_input("Pilih nomor file cookie yang ingin digunakan: ");

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
	match mode {
		Mode::Normal => {
			println!("Contoh input: \npromotion_id: 856793882394624, \nSignature: 8e8a4ced8d6905570114f163a08a15de55c3fed560f8a3a8a25e6e179783b480");
			let promotion_id = get_user_input("Masukkan nilai promotion_id: ");

			let signature = get_user_input("Masukkan nilai Signature: ");
			
			let task_time_str = opt.time.clone().unwrap_or_else(|| get_user_input("Enter task time (HH:MM:SS.NNNNNNNNN): "));
			let task_time_dt = parse_task_time(&task_time_str)?;

			// Process HTTP with common function
			countdown_to_task(task_time_dt).await;
			some_function(&promotion_id, &signature, &cookie_content).await?;
		}
		Mode::Collection => {
			println!("Contoh input: collection_id: 12923214728");
			let start = get_user_input("Masukkan nilai collection_id: ");
			let task_time_str = opt.time.clone().unwrap_or_else(|| get_user_input("Enter task time (HH:MM:SS.NNNNNNNNN): "));
			let task_time_dt = parse_task_time(&task_time_str)?;
			// Process HTTP with common function
			countdown_to_task(task_time_dt).await;
			let (promotion_id, signature) = prepare::some_function(&start, &cookie_content).await?;
			println!("promotion_id : {}", promotion_id);
			println!("signature	: {}", signature);
			some_function(&promotion_id, &signature, &cookie_content).await?;
		}
	}
	println!("\nTask completed! Current time: {}", Local::now().format("%H:%M:%S.%3f"));
	Ok(())
}

fn select_mode() -> Mode {
	loop {
		println!("Pilih mode:");
		println!("1. Normal");
		println!("2. Collection");

		print!("Masukkan pilihan (1/2): ");
		io::stdout().flush().unwrap();

		let mut input = String::new();
		io::stdin().read_line(&mut input).expect("Gagal membaca baris");

		match input.trim() {
			"1" => return Mode::Normal,
			"2" => return Mode::Collection,
			_ => println!("Pilihan tidak valid, coba lagi."),
		}
	}
}
async fn check_and_adjust_time(task_time_dt: NaiveDateTime) -> NaiveDateTime {
	let mut updated_task_time_dt = task_time_dt;
	let current_time = Local::now().naive_local();
	let time_until_task = updated_task_time_dt.signed_duration_since(current_time);

	if time_until_task < Duration::zero() {
		// Jika waktu sudah berlalu, tawarkan untuk menyesuaikan waktu
		println!("Waktu yang dimasukkan telah berlalu.");
		println!("Apakah Anda ingin menyetel waktu untuk besok? (yes/no): ");
		
		let mut input = String::new();
		io::stdin().read_line(&mut input).expect("Gagal membaca baris");

		match input.trim().to_lowercase().as_str() {
			"yes" | "y" => {
				// Tambahkan satu hari ke waktu target
				updated_task_time_dt += Duration::days(1);
				println!("Waktu telah disesuaikan untuk hari berikutnya: {}", updated_task_time_dt);
			}
			_ => println!("Pengaturan waktu tidak diubah."),
		}
	}

	updated_task_time_dt
}

async fn countdown_to_task(task_time_dt: NaiveDateTime) {
	let task_time_dt = check_and_adjust_time(task_time_dt).await;

	loop {
		let current_time = Local::now().naive_local();
		let time_until_task = task_time_dt.signed_duration_since(current_time);

		if time_until_task <= Duration::zero() {
			println!("\nTask completed! Current time: {}", current_time.format("%H:%M:%S.%3f"));
			tugas_utama();
			break;
		}

		let formatted_time = format_duration(time_until_task);
		print!("\r{}", formatted_time);
		io::stdout().flush().unwrap();

		tokio::time::sleep(tokio::time::Duration::from_millis(1)).await;
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
fn parse_task_time(task_time_str: &str) -> Result<NaiveDateTime, Box<dyn std::error::Error>> {
	let today = Local::now().date_naive();
	let dt = NaiveDateTime::parse_from_str(&format!("{} {}", today.format("%Y-%m-%d"), task_time_str), "%Y-%m-%d %H:%M:%S%.f")?;
	Ok(dt)
}