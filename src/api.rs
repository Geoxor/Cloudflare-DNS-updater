use serde::{Deserialize, Serialize};

/// Struct for the current IP address
#[derive(Serialize, Deserialize)]
pub struct CurrentIP {
  /// The current IP address
  pub ip: String,
}

/// Struct for the Cloudflares response
#[derive(Serialize, Deserialize, Debug)]
pub struct CloudFlareResult {
  /// The Cloudflare response containing the array of records
  pub result: Vec<DNSRecordResult>,
}
/// The struct for the DNS record
#[derive(Serialize, Deserialize, Debug)]
pub struct DNSRecordResult {
  /// The DNS record ID
  pub id: String,
  /// The DNS record name
  pub name: String,
  /// The DNS record IP address
  pub content: String,
  /// Is the DNS record locked?
  pub locked: bool,
  /// Is the DNS record proxied?
  pub proxied: bool,
  /// The DNS record ttl
  pub ttl: u32,
  /// The DNS record zone ID
  pub zone_id: String,
  /// The DNS record modified date
  pub modified_on: String,
}

/// Struct for the record update template
#[derive(Serialize, Deserialize, Debug)]
pub struct UpdateRecord {
  /// The DNS record type
  pub r#type: String,
  /// The DNS record name
  pub name: String,
  /// The DNS record IP address
  pub content: String,
  /// The DNS record ttl
  pub ttl: u32,
  /// Is the DNS record proxied?
  pub proxied: bool,
}

const IP_ADDRESS_URL: &str = "https://api.ipify.org?format=json";
const CF_BASE_URL: &str = "https://api.cloudflare.com/client/v4/zones/";

/// Fetches the current public IP address
///
/// # Example
/// ```
/// use cloudflare_dns_updater::api;
///
/// let ip = api::get_current_ip().await?;
/// println!("{}", ip);
/// ```
///
/// # Errors
/// Returns an error if the request fails
///
/// # Returns
/// The current public IP address
///
/// # Remarks
/// The IP address is fetched from https://api.ipify.org
///
/// # See Also
/// * [https://api.ipify.org](https://api.ipify.org)
pub async fn get_current_ip() -> Result<String, reqwest::Error> {
  let response = reqwest::get(IP_ADDRESS_URL).await?;
  let cur_ip: CurrentIP = response.json().await?;
  Ok(cur_ip.ip)
}

/// Fetches the DNS records for the given zone
///
/// # Example
/// ```
/// use cloudflare_dns_updater::api;
///
/// let records = api::get_record_ip(
///  &vec!["test.domain.com", "test2.example.com"],
/// "cl0udfl4r3z0n31d",
/// "YOUR_API_KEY_HERE"
/// ).await?;
/// ```
///
/// # Errors
/// Returns an error if the request fails
///
/// Returns an error if the JSON is not a valid
///
/// # Returns
/// The DNS records for the given zone that match the given records
///
/// # Remarks
/// The DNS records are fetched from https://api.cloudflare.com/client/v4/zones/
///
/// # See Also
/// * [https://api.cloudflare.com/client/v4/zones/](https://api.cloudflare.com/client/v4/zones/)
pub async fn get_record_ip(
  records: &Vec<String>,
  zone: &str,
  auth_key: &str,
) -> Result<CloudFlareResult, Box<dyn std::error::Error>> {
  let client = reqwest::Client::new();
  let url = format!("{}{}/dns_records?type=A", CF_BASE_URL, zone);
  let res = client
    .get(url)
    .header("Authorization", format!("Bearer {}", auth_key))
    .send()
    .await?
    .text()
    .await?;
  let mut results: CloudFlareResult = serde_json::from_str(&res)?;
  results
    .result
    .retain(|record| records.contains(&record.name));
  Ok(results)
}

/// Updates the given DNS records IP address
///
/// # Example
/// ```
/// use cloudflare_dns_updater::api;
///
/// let records = api::get_record_ip(
/// "test2.example.com",
/// "102.46.132.14",
/// "YOUR_API_KEY_HERE"
/// ).await?;
/// ```
///
/// # Returns
/// * `Ok(())` - If the IP address was updated successfully
/// * `Err(e)` - If the IP address could not be updated
///
/// # Remarks
/// The DNS records are updated from https://api.cloudflare.com/client/v4/zones/
///
/// # See Also
/// * [https://api.cloudflare.com/client/v4/zones/](https://api.cloudflare.com/client/v4/zones/)
pub async fn update_record(
  record: &DNSRecordResult,
  ip: &str,
  auth_key: &str,
) -> Result<(), Box<dyn std::error::Error>> {
  let client = reqwest::Client::new();
  let url = format!(
    "{}{}/dns_records/{}",
    CF_BASE_URL, record.zone_id, record.id
  );
  client
    .put(url)
    .header("Authorization", format!("Bearer {}", auth_key))
    .json(&UpdateRecord {
      r#type: "A".to_string(),
      name: record.name.to_string(),
      content: ip.to_string(),
      ttl: record.ttl,
      proxied: record.proxied,
    })
    .send()
    .await?
    .text()
    .await?;
  Ok(())
}
