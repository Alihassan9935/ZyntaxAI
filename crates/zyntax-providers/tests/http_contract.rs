use std::time::Duration;
use tokio_util::sync::CancellationToken;
use wiremock::matchers::{header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};
use zyntax_core::{
    auto_language, builtin_persona, Prompt, PromptSpec, ProviderId, ProviderProfile, Speed,
    DEFAULT_PERSONA_ID,
};
use zyntax_providers::{build, CompletionRequest, ProviderError};

fn prompt() -> Prompt {
    let persona = builtin_persona(DEFAULT_PERSONA_ID).expect("default persona");
    let language = auto_language();
    PromptSpec {
        persona: &persona,
        language: &language,
        translate: false,
        speed: Speed::Normal,
    }
    .build("i dont think its correct")
    .expect("prompt builds")
}

fn request(model: &str) -> CompletionRequest {
    CompletionRequest {
        model: model.to_owned(),
        prompt: prompt(),
    }
}

fn profile(id: ProviderId, base_url: &str) -> ProviderProfile {
    ProviderProfile {
        id,
        base_url: Some(base_url.to_owned()),
        model: id.default_model().to_owned(),
    }
}

fn gemini_success() -> serde_json::Value {
    serde_json::json!({
        "candidates": [{
            "content": { "parts": [{ "text": "I don't think it's correct." }] },
            "finishReason": "STOP"
        }],
        "usageMetadata": { "promptTokenCount": 42, "candidatesTokenCount": 9 }
    })
}

#[tokio::test]
async fn gemini_returns_corrected_text_and_usage() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/models/gemini-2.5-flash:generateContent"))
        .and(header("x-goog-api-key", "test-key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(gemini_success()))
        .expect(1)
        .mount(&server)
        .await;

    let provider = build(
        &profile(ProviderId::Gemini, &server.uri()),
        Some("test-key".to_owned()),
    )
    .expect("builds");

    let completion = provider
        .complete(&request("gemini-2.5-flash"), &CancellationToken::new())
        .await
        .expect("succeeds");

    assert_eq!(completion.text, "I don't think it's correct.");
    assert_eq!(completion.usage.input_tokens, 42);
    assert_eq!(completion.usage.output_tokens, 9);
}

#[tokio::test]
async fn gemini_rejects_a_bad_key_without_retrying() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(401).set_body_json(serde_json::json!({
            "error": { "message": "API key not valid" }
        })))
        .expect(1)
        .mount(&server)
        .await;

    let provider = build(
        &profile(ProviderId::Gemini, &server.uri()),
        Some("bad-key".to_owned()),
    )
    .expect("builds");

    let error = provider
        .complete(&request("gemini-2.5-flash"), &CancellationToken::new())
        .await
        .expect_err("must fail");

    assert!(matches!(error, ProviderError::Auth { .. }));
    assert!(!error.is_retryable());
}

#[tokio::test]
async fn gemini_retries_server_errors_then_gives_up() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(503))
        .expect(3)
        .mount(&server)
        .await;

    let provider = build(
        &profile(ProviderId::Gemini, &server.uri()),
        Some("test-key".to_owned()),
    )
    .expect("builds");

    let error = provider
        .complete(&request("gemini-2.5-flash"), &CancellationToken::new())
        .await
        .expect_err("must fail");

    assert!(matches!(error, ProviderError::Server { status: 503, .. }));
}

#[tokio::test]
async fn gemini_recovers_when_a_retry_succeeds() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(500))
        .up_to_n_times(1)
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(200).set_body_json(gemini_success()))
        .expect(1)
        .mount(&server)
        .await;

    let provider = build(
        &profile(ProviderId::Gemini, &server.uri()),
        Some("test-key".to_owned()),
    )
    .expect("builds");

    let completion = provider
        .complete(&request("gemini-2.5-flash"), &CancellationToken::new())
        .await
        .expect("second attempt succeeds");

    assert_eq!(completion.text, "I don't think it's correct.");
}

#[tokio::test]
async fn gemini_reports_rate_limiting_with_the_providers_wait() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .respond_with(
            ResponseTemplate::new(429)
                .insert_header("retry-after", "1")
                .set_body_json(serde_json::json!({ "error": { "message": "quota" } })),
        )
        .mount(&server)
        .await;

    let provider = build(
        &profile(ProviderId::Gemini, &server.uri()),
        Some("test-key".to_owned()),
    )
    .expect("builds");

    let error = provider
        .complete(&request("gemini-2.5-flash"), &CancellationToken::new())
        .await
        .expect_err("must fail");

    match error {
        ProviderError::RateLimited { retry_after } => {
            assert_eq!(retry_after, Some(Duration::from_secs(1)));
        }
        other => panic!("expected a rate limit, got {other:?}"),
    }
}

#[tokio::test]
async fn gemini_reports_a_malformed_payload_rather_than_panicking() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(200).set_body_string("<html>not json</html>"))
        .mount(&server)
        .await;

    let provider = build(
        &profile(ProviderId::Gemini, &server.uri()),
        Some("test-key".to_owned()),
    )
    .expect("builds");

    let error = provider
        .complete(&request("gemini-2.5-flash"), &CancellationToken::new())
        .await
        .expect_err("must fail");

    assert!(matches!(error, ProviderError::Malformed(_)));
}

#[tokio::test]
async fn gemini_surfaces_truncation_instead_of_a_half_sentence() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "candidates": [{
                "content": { "parts": [{ "text": "I don't think it's cor" }] },
                "finishReason": "MAX_TOKENS"
            }]
        })))
        .mount(&server)
        .await;

    let provider = build(
        &profile(ProviderId::Gemini, &server.uri()),
        Some("test-key".to_owned()),
    )
    .expect("builds");

    let error = provider
        .complete(&request("gemini-2.5-flash"), &CancellationToken::new())
        .await
        .expect_err("must fail");

    assert!(matches!(error, ProviderError::Truncated));
}

#[tokio::test]
async fn gemini_lists_only_models_that_can_generate() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/models"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "models": [
                {
                    "name": "models/gemini-2.5-flash",
                    "displayName": "Gemini 2.5 Flash",
                    "supportedGenerationMethods": ["generateContent"]
                },
                {
                    "name": "models/text-embedding-004",
                    "displayName": "Text Embedding",
                    "supportedGenerationMethods": ["embedContent"]
                }
            ]
        })))
        .mount(&server)
        .await;

    let provider = build(
        &profile(ProviderId::Gemini, &server.uri()),
        Some("test-key".to_owned()),
    )
    .expect("builds");

    let models = provider.list_models().await.expect("lists");

    assert_eq!(models.len(), 1, "embedding models cannot correct text");
    assert_eq!(models[0].id, "gemini-2.5-flash");
    assert_eq!(models[0].label, "Gemini 2.5 Flash");
}

#[tokio::test]
async fn a_cancelled_request_stops_rather_than_completing() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_delay(Duration::from_secs(30))
                .set_body_json(gemini_success()),
        )
        .mount(&server)
        .await;

    let provider = build(
        &profile(ProviderId::Gemini, &server.uri()),
        Some("test-key".to_owned()),
    )
    .expect("builds");

    let cancel = CancellationToken::new();
    let trigger = cancel.clone();
    tokio::spawn(async move {
        tokio::time::sleep(Duration::from_millis(100)).await;
        trigger.cancel();
    });

    let error = provider
        .complete(&request("gemini-2.5-flash"), &cancel)
        .await
        .expect_err("must be cancelled");

    assert!(matches!(error, ProviderError::Cancelled));
}

#[tokio::test]
async fn openai_compatible_returns_corrected_text_and_usage() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/chat/completions"))
        .and(header("authorization", "Bearer sk-test"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "choices": [{
                "message": { "role": "assistant", "content": "I don't think it's correct." },
                "finish_reason": "stop"
            }],
            "usage": { "prompt_tokens": 30, "completion_tokens": 8 }
        })))
        .expect(1)
        .mount(&server)
        .await;

    let provider = build(
        &profile(ProviderId::OpenAiCompatible, &server.uri()),
        Some("sk-test".to_owned()),
    )
    .expect("builds");

    let completion = provider
        .complete(&request("gpt-4o-mini"), &CancellationToken::new())
        .await
        .expect("succeeds");

    assert_eq!(completion.text, "I don't think it's correct.");
    assert_eq!(completion.usage.input_tokens, 30);
}

#[tokio::test]
async fn openai_compatible_sorts_the_model_list() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/models"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": [{ "id": "zeta" }, { "id": "alpha" }, { "id": "mu" }]
        })))
        .mount(&server)
        .await;

    let provider = build(
        &profile(ProviderId::OpenAiCompatible, &server.uri()),
        Some("sk-test".to_owned()),
    )
    .expect("builds");

    let models = provider.list_models().await.expect("lists");
    let ids: Vec<&str> = models.iter().map(|m| m.id.as_str()).collect();
    assert_eq!(ids, ["alpha", "mu", "zeta"]);
}

#[tokio::test]
async fn openai_compatible_names_an_unknown_model() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(404).set_body_json(serde_json::json!({
            "error": { "message": "The model 'gpt-9' does not exist" }
        })))
        .mount(&server)
        .await;

    let provider = build(
        &profile(ProviderId::OpenAiCompatible, &server.uri()),
        Some("sk-test".to_owned()),
    )
    .expect("builds");

    let error = provider
        .complete(&request("gpt-9"), &CancellationToken::new())
        .await
        .expect_err("must fail");

    assert!(error.to_string().contains("gpt-9"), "got {error}");
}

#[tokio::test]
async fn ollama_works_without_an_api_key() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/api/chat"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "message": { "role": "assistant", "content": "I don't think it's correct." },
            "done": true,
            "done_reason": "stop",
            "prompt_eval_count": 25,
            "eval_count": 8
        })))
        .expect(1)
        .mount(&server)
        .await;

    let provider = build(&profile(ProviderId::Ollama, &server.uri()), None).expect("builds");

    let completion = provider
        .complete(&request("llama3.2"), &CancellationToken::new())
        .await
        .expect("succeeds");

    assert_eq!(completion.text, "I don't think it's correct.");
    assert_eq!(completion.usage.output_tokens, 8);
}

#[tokio::test]
async fn ollama_lists_installed_models() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/tags"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "models": [{ "name": "qwen2.5:7b" }, { "name": "llama3.2:latest" }]
        })))
        .mount(&server)
        .await;

    let provider = build(&profile(ProviderId::Ollama, &server.uri()), None).expect("builds");
    let models = provider.list_models().await.expect("lists");

    assert_eq!(models.len(), 2);
    assert_eq!(models[0].id, "llama3.2:latest", "sorted");
}

#[tokio::test]
async fn a_dead_ollama_reports_a_network_error_with_a_useful_remedy() {
    let provider = build(&profile(ProviderId::Ollama, "http://127.0.0.1:1"), None).expect("builds");

    let error = provider
        .complete(&request("llama3.2"), &CancellationToken::new())
        .await
        .expect_err("must fail");

    assert!(matches!(error, ProviderError::Network { .. }));
    assert!(error.remedy().contains("ollama serve"));
}
