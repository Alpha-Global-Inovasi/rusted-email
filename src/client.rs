use crate::config::{EmailConfig, EmailProviderType};
use crate::dto::request::{EmailContent, EmailRequest};
use crate::dto::response::EmailResponse;
use crate::error::EmailError;
use crate::provider::{EmailProvider, mock::MockProvider, smtp::SmtpProvider};
use crate::template::engine::TemplateEngine;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Clone)]
pub struct EmailClient {
    provider: Arc<dyn EmailProvider + Send + Sync>,
    template_engine: Arc<Mutex<TemplateEngine>>,
}

impl EmailClient {
    pub async fn new(config: EmailConfig) -> Result<Self, EmailError> {
        let provider: Arc<dyn EmailProvider + Send + Sync> = match config.provider {
            EmailProviderType::Smtp => {
                let smtp_config = config
                    .smtp_config
                    .ok_or(EmailError::ConfigError("SMTP config missing".to_string()))?;
                Arc::new(SmtpProvider::new(
                    smtp_config,
                    config.from_email.clone(),
                    config.from_name.clone(),
                )?)
            }
            EmailProviderType::Mock => Arc::new(MockProvider),
        };

        Ok(Self {
            provider,
            template_engine: Arc::new(Mutex::new(TemplateEngine::new())),
        })
    }

    pub async fn register_template(&self, name: &str, template: &str) -> Result<(), EmailError> {
        let mut engine = self.template_engine.lock().await;
        engine.register_template(name, template)
    }

    pub async fn send(&self, mut req: EmailRequest) -> Result<EmailResponse<String>, EmailError> {
        // Render template if needed
        if let EmailContent::Template { name, data } = &req.content {
            let engine = self.template_engine.lock().await;
            let html = engine.render(name, data)?;
            let text_name = format!("{name}.txt");
            req.content = if engine.has_template(&text_name) {
                EmailContent::Alternative {
                    html,
                    text: engine.render(&text_name, data)?,
                }
            } else {
                EmailContent::Html(html)
            };
        }

        self.provider.send(req).await?;

        Ok(EmailResponse::success(
            "Email sent successfully",
            "ok".to_string(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::EmailProviderType;

    async fn mock_client() -> EmailClient {
        EmailClient::new(EmailConfig {
            provider: EmailProviderType::Mock,
            from_email: "from@example.com".to_string(),
            from_name: "Example".to_string(),
            smtp_config: None,
        })
        .await
        .unwrap()
    }

    #[tokio::test]
    async fn send_renders_registered_template() {
        let client = mock_client().await;
        client
            .register_template("hello", "Hi {{name}}")
            .await
            .unwrap();

        let req = EmailRequest::builder()
            .to("to@example.com")
            .subject("Hello")
            .template("hello", serde_json::json!({ "name": "Kanda" }))
            .build()
            .unwrap();

        assert!(client.send(req).await.unwrap().status);
    }

    #[tokio::test]
    async fn send_renders_a_text_template_when_registered() {
        let client = mock_client().await;
        client
            .register_template("hi", "<p>Hi {{name}}</p>")
            .await
            .unwrap();
        client
            .register_template("hi.txt", "Hi {{name}} in text")
            .await
            .unwrap();
        let req = EmailRequest::builder()
            .to("to@example.com")
            .subject("Hi")
            .template("hi", serde_json::json!({ "name": "Ayu" }))
            .build()
            .unwrap();
        assert!(client.send(req).await.unwrap().status);
    }

    #[tokio::test]
    async fn send_rejects_unknown_template() {
        let client = mock_client().await;

        let req = EmailRequest::builder()
            .to("to@example.com")
            .subject("Hello")
            .template("missing", serde_json::json!({}))
            .build()
            .unwrap();

        assert!(matches!(
            client.send(req).await,
            Err(EmailError::TemplateError(_))
        ));
    }
}
