use reqwest::Client;

use super::models::{ActivateLicenseRequest, LicenseResponse, VerifyLicenseRequest};

#[derive(Clone)]
pub struct LicenseClient {
    client: Client,
    base_url: String,
}

impl LicenseClient {
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            client: Client::new(),
            base_url: base_url.into(),
        }
    }

    pub async fn activate(&self, req: ActivateLicenseRequest) -> Result<LicenseResponse, String> {
        let url = format!("{}/api/license/activate", self.base_url);

        let res = self
            .client
            .post(url)
            .json(&req)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if !res.status().is_success() {
            let body = res.text().await.unwrap_or_default();

            return Err(format!("activation failed: {}", body));
        }

        res.json::<LicenseResponse>()
            .await
            .map_err(|e| e.to_string())
    }

    pub async fn verify(&self, req: VerifyLicenseRequest) -> Result<LicenseResponse, String> {
        let url = format!("{}/api/license/verify", self.base_url);

        let res = self
            .client
            .post(url)
            .json(&req)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if !res.status().is_success() {
            let body = res.text().await.unwrap_or_default();

            return Err(format!("verification failed: {}", body));
        }

        res.json::<LicenseResponse>()
            .await
            .map_err(|e| e.to_string())
    }

    pub async fn get_portal_url(&self, email: &str) -> Result<String, String> {
        let url = format!("{}/api/portal", self.base_url);

        let res = self
            .client
            .post(url)
            .json(&serde_json::json!({ "email": email }))
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if !res.status().is_success() {
            let body = res.text().await.unwrap_or_default();

            return Err(format!("portal session failed: {}", body));
        }

        let json: serde_json::Value = res.json().await.map_err(|e| e.to_string())?;

        json["portal_url"]
            .as_str()
            .map(String::from)
            .ok_or_else(|| "missing portal_url in response".to_string())
    }
}
