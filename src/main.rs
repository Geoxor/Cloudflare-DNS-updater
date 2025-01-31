use api::CloudFlareResult;
use config_loader::Config;

mod api;
mod config_loader;

const VERSION: &'static str = env!("CARGO_PKG_VERSION");

/// The main function of the program
#[tokio::main]
async fn main() {
  println!("Cloudflare IP updater v{}", VERSION);
  print!("Loading config... ");
  let config =
    config_loader::load_config(handle_args().as_str()).expect("\nConfigLoadFailException");
  println!("Loaded!");
  loop {
    match check_and_update_ip(&config).await {
      Ok(()) => {}
      Err(e) => println!("\nError: {}", e),
    }
    std::thread::sleep(std::time::Duration::from_secs(config.update_threshold));
  }
}

/// Checks the zones and their flagged records for IP address changes and updates them.
///
///  # Arguments
/// * `config` - The configuration to use
///
/// # Returns
/// * `Ok(())` - If the IP addresses were updated successfully
/// * `Err(e)` - If the IP addresses could not be updated
async fn check_and_update_ip(config: &Config) -> Result<(), Box<dyn std::error::Error>> {
  print!("\nGetting current IP address");
  let cur_ip = api::get_current_ip().await?;
  println!(" - {}\n", cur_ip);
  for k in 0..config.keys.len() {
    println!("Updating zones for key {}", config.keys[k].auth_key);
    for z in 0..config.keys[k].zones.len() {
      println!(
        "Updating records for zone {}",
        config.keys[k].zones[z].zone_id
      );
      let record_ips: CloudFlareResult = api::get_record_ip(
        &config.keys[k].zones[z].records,
        &config.keys[k].zones[z].zone_id,
        &config.keys[k].auth_key,
      )
      .await?;
      for i in 0..config.keys[k].zones[z].records.len() {
        if !record_ips.result.get(i).is_none() && !record_ips.result[i].locked {
          if cur_ip != record_ips.result[i].content {
            print!(
              "Updating record {} from {} to {}",
              record_ips.result[i].name, record_ips.result[i].content, cur_ip
            );
            match api::update_record(&record_ips.result[i], &cur_ip, &config.keys[k].auth_key).await
            {
              Ok(()) => println!(" - Record updated"),
              Err(e) => println!(" - Error: {}", e),
            }
          } else {
            println!("Record {} is up to date", record_ips.result[i].name);
          }
        }
      }
      println!("Done updating zone")
    }
    println!("Done updating keys zones")
  }
  Ok(())
}

/// Handles the input arguments.
/// Currently only custom config parameter is supported
///
/// # Returns
/// * `String` - Path to the custom config file
///
/// # Examples
///
/// ```
/// use cloudflare_dns_updater::handle_args;
/// use cloudflare_dns_updater::config_loader::load_config;
///
/// let config_path = handle_args();
/// let config = load_config(config_path.as_str()).expect("Failed to load config!");
/// ```
///
/// # Errors
/// * `std::env::VarError` - If the environment variable could not be read or parsed
/// * `std::env::VarError` - If the environment variable is not set
fn handle_args() -> String {
  let mut config_path: Option<&str> = None;
  let args: Vec<String> = std::env::args().collect();
  let mut index: usize = 0;
  while args.len() > index {
    let arg: &str = &args[index];
    match arg {
      "-c" => {
        index = index + 1;
        if index < args.len() {
          config_path = Some(&args[index]);
          index = index + 1;
          println!("Using custom config path: {:?}", config_path);
        }
      }
      _ => index = index + 1,
    }
  }
  return config_path.unwrap_or("config.json").to_string();
}
