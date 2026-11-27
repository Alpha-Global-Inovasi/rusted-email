use crate::error::EmailError;
use handlebars::Handlebars;
use serde_json::Value;

pub struct TemplateEngine {
    registry: Handlebars<'static>,
}

impl Default for TemplateEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl TemplateEngine {
    pub fn new() -> Self {
        Self {
            registry: Handlebars::new(),
        }
    }

    pub fn register_template(
        &mut self,
        name: &str,
        template_string: &str,
    ) -> Result<(), EmailError> {
        self.registry
            .register_template_string(name, template_string)
            .map_err(|e| EmailError::TemplateError(e.to_string()))
    }

    pub fn has_template(&self, name: &str) -> bool {
        self.registry.has_template(name)
    }

    pub fn render(&self, name: &str, data: &Value) -> Result<String, EmailError> {
        self.registry
            .render(name, data)
            .map_err(|e| EmailError::TemplateError(e.to_string()))
    }
}
