use crate::model::ParticipantCardData;
use reqwest::{Client, Url};
use serde::{de::DeserializeOwned, Deserialize};
use std::time::Duration;

const PARTICIPANTS_PATH: &str = "/api/v1/participants";
const RECOMMENDATIONS_PATH: &str = "/api/v1/recommendations";
const APPLY_RECOMMENDATION_PATH: &str = "/api/v1/recommendations/{participant_id}/apply";

#[derive(Debug, Deserialize)]
struct ApiEnvelope<T> {
    ok: bool,
    data: Option<T>,
    message: Option<String>,
}

#[derive(Debug, Deserialize)]
struct RecommendationIdentity {
    id: i64,
}

#[derive(Debug, Deserialize)]
struct ApiIndex {
    endpoints: Vec<ApiEndpoint>,
}

#[derive(Debug, Deserialize)]
struct ApiEndpoint {
    method: String,
    path: String,
}

impl ApiIndex {
    fn supports(&self, method: &str, path: &str) -> bool {
        self.endpoints
            .iter()
            .any(|endpoint| endpoint.method.eq_ignore_ascii_case(method) && endpoint.path == path)
    }

    fn validate_participant_access(&self) -> Result<(), String> {
        if self.supports("GET", PARTICIPANTS_PATH) {
            Ok(())
        } else {
            Err(
                "当前 API Key 没有读取参与者的权限。请让管理员为该系统用户开放“参与者”页面，并配置至少一个可见参与者。"
                    .to_string(),
            )
        }
    }

    fn can_apply_recommendations(&self) -> bool {
        self.supports("GET", RECOMMENDATIONS_PATH)
            && self.supports("POST", APPLY_RECOMMENDATION_PATH)
    }
}

pub fn normalize_base_url(value: &str) -> Result<String, String> {
    let mut url = Url::parse(value.trim()).map_err(|_| "服务地址格式无效".to_string())?;
    if !matches!(url.scheme(), "http" | "https") || url.host_str().is_none() {
        return Err("服务地址必须是有效的 HTTP 或 HTTPS 地址".to_string());
    }
    if !url.username().is_empty() || url.password().is_some() {
        return Err("服务地址不能包含用户名或密码".to_string());
    }
    url.set_query(None);
    url.set_fragment(None);
    Ok(url.as_str().trim_end_matches('/').to_string())
}

fn build_client() -> Result<Client, String> {
    Client::builder()
        .connect_timeout(Duration::from_secs(8))
        .timeout(Duration::from_secs(20))
        .build()
        .map_err(|error| format!("创建网络客户端失败：{error}"))
}

async fn fetch_collection<T: DeserializeOwned>(
    client: &Client,
    endpoint: String,
    token: &str,
    missing_data: &str,
) -> Result<T, String> {
    let response = client
        .get(endpoint)
        .bearer_auth(token)
        .header("Accept", "application/json")
        .send()
        .await
        .map_err(|error| format!("无法连接 Sub2Pool：{error}"))?;
    let status = response.status();
    let body = response
        .text()
        .await
        .map_err(|error| format!("读取 Sub2Pool 响应失败：{error}"))?;
    let envelope: ApiEnvelope<T> = serde_json::from_str(&body)
        .map_err(|_| format!("Sub2Pool 返回了无效响应（HTTP {status}）"))?;
    if !status.is_success() || !envelope.ok {
        return Err(envelope
            .message
            .unwrap_or_else(|| format!("Sub2Pool 请求失败（HTTP {status}）")));
    }
    envelope.data.ok_or_else(|| missing_data.to_string())
}

pub async fn fetch_card_data(
    base_url: &str,
    token: &str,
) -> Result<(Vec<ParticipantCardData>, Vec<i64>), String> {
    let client = build_client()?;
    let api_index: ApiIndex = fetch_collection(
        &client,
        format!("{base_url}/api/v1"),
        token,
        "Sub2Pool 响应缺少 API 能力信息",
    )
    .await?;
    api_index.validate_participant_access()?;
    let participants = fetch_collection(
        &client,
        format!("{base_url}{PARTICIPANTS_PATH}"),
        token,
        "Sub2Pool 响应缺少参与者数据",
    )
    .await?;
    let recommendations: Vec<RecommendationIdentity> = if api_index.can_apply_recommendations() {
        fetch_collection(
            &client,
            format!("{base_url}{RECOMMENDATIONS_PATH}"),
            token,
            "Sub2Pool 响应缺少待应用建议数据",
        )
        .await?
    } else {
        Vec::new()
    };
    Ok((
        participants,
        recommendations.into_iter().map(|item| item.id).collect(),
    ))
}

pub async fn apply_participant_recommendation(
    base_url: &str,
    token: &str,
    participant_id: i64,
) -> Result<(), String> {
    let endpoint = format!("{base_url}/api/v1/recommendations/{participant_id}/apply");
    let response = build_client()?
        .post(endpoint)
        .bearer_auth(token)
        .header("Accept", "application/json")
        .send()
        .await
        .map_err(|error| format!("无法连接 Sub2Pool：{error}"))?;
    let status = response.status();
    let body = response
        .text()
        .await
        .map_err(|error| format!("读取 Sub2Pool 响应失败：{error}"))?;
    let envelope: ApiEnvelope<serde_json::Value> = serde_json::from_str(&body)
        .map_err(|_| format!("Sub2Pool 返回了无效响应（HTTP {status}）"))?;
    if !status.is_success() || !envelope.ok {
        return Err(envelope
            .message
            .unwrap_or_else(|| format!("应用额度建议失败（HTTP {status}）")));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        normalize_base_url, ApiEndpoint, ApiIndex, APPLY_RECOMMENDATION_PATH, PARTICIPANTS_PATH,
        RECOMMENDATIONS_PATH,
    };

    #[test]
    fn normalizes_subpath_without_leaking_url_credentials() {
        assert_eq!(
            normalize_base_url(" https://pool.example.com/sub2pool/?ignored=1 ").unwrap(),
            "https://pool.example.com/sub2pool"
        );
        assert!(normalize_base_url("https://admin:secret@pool.example.com").is_err());
        assert!(normalize_base_url("file:///tmp/sub2pool").is_err());
    }

    fn api_index(endpoints: &[(&str, &str)]) -> ApiIndex {
        ApiIndex {
            endpoints: endpoints
                .iter()
                .map(|(method, path)| ApiEndpoint {
                    method: (*method).to_string(),
                    path: (*path).to_string(),
                })
                .collect(),
        }
    }

    #[test]
    fn system_user_cards_stay_read_only_from_discovered_capabilities() {
        let system_user = api_index(&[("GET", PARTICIPANTS_PATH), ("GET", RECOMMENDATIONS_PATH)]);
        assert!(system_user.validate_participant_access().is_ok());
        assert!(!system_user.can_apply_recommendations());

        let administrator = api_index(&[
            ("GET", PARTICIPANTS_PATH),
            ("GET", RECOMMENDATIONS_PATH),
            ("POST", APPLY_RECOMMENDATION_PATH),
        ]);
        assert!(administrator.can_apply_recommendations());
    }

    #[test]
    fn participant_page_permission_is_required_for_subcard() {
        let dashboard_only = api_index(&[("GET", RECOMMENDATIONS_PATH)]);
        assert_eq!(
            dashboard_only.validate_participant_access().unwrap_err(),
            "当前 API Key 没有读取参与者的权限。请让管理员为该系统用户开放“参与者”页面，并配置至少一个可见参与者。"
        );
    }
}
