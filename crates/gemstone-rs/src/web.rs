use crate::{Config, Error, Result, Session, SessionWorkerPool, Value};

pub const DEFAULT_SERVICE_NAME: &str = "gemstone-rs service";

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct JsonResponse {
    pub status: u16,
    pub body: String,
}

impl JsonResponse {
    pub fn ok(body: impl Into<String>) -> Self {
        Self {
            status: 200,
            body: body.into(),
        }
    }

    pub fn error(status: u16, message: impl AsRef<str>) -> Self {
        Self {
            status,
            body: format!(r#"{{"error":"{}"}}"#, json_escape(message.as_ref())),
        }
    }
}

pub fn index_response(name: &str) -> JsonResponse {
    JsonResponse::ok(format!(
        r#"{{"name":"{}","endpoints":{{"local":"/health/local","gemstone":"/health/gemstone"}}}}"#,
        json_escape(name)
    ))
}

pub fn local_health_response() -> JsonResponse {
    JsonResponse::ok(r#"{"ok":true}"#)
}

pub fn bad_request_response() -> JsonResponse {
    JsonResponse::error(400, "bad request")
}

pub fn method_not_allowed_response() -> JsonResponse {
    JsonResponse::error(405, "method not allowed")
}

pub fn not_found_response() -> JsonResponse {
    JsonResponse::error(404, "not found")
}

pub fn gemstone_health_response(pool: &SessionWorkerPool) -> JsonResponse {
    gemstone_health_response_from_result(gemstone_health_value(pool))
}

pub fn gemstone_health_response_once(config: Config) -> JsonResponse {
    gemstone_health_response_from_result(gemstone_health_value_once(config))
}

pub fn gemstone_health_response_from_result(result: Result<i64>) -> JsonResponse {
    match result {
        Ok(value) => JsonResponse::ok(format!(r#"{{"result":{value}}}"#)),
        Err(err) => JsonResponse::error(500, err.to_string()),
    }
}

pub fn gemstone_health_value(pool: &SessionWorkerPool) -> Result<i64> {
    smallint_health_value(pool.eval("3 + 4")?)
}

pub fn gemstone_health_value_once(config: Config) -> Result<i64> {
    let mut session = Session::login(config)?;
    let value = smallint_health_value(session.eval("3 + 4")?);
    let logout = session.logout();
    let value = value?;
    logout?;
    Ok(value)
}

fn smallint_health_value(value: Value) -> Result<i64> {
    match value {
        Value::SmallInt(value) => Ok(value),
        other => Err(Error::UnexpectedType {
            expected: "SmallInt",
            actual: format!("{other:?}"),
        }),
    }
}

pub fn json_escape(value: &str) -> String {
    let mut output = String::new();
    for ch in value.chars() {
        match ch {
            '"' => output.push_str("\\\""),
            '\\' => output.push_str("\\\\"),
            '\n' => output.push_str("\\n"),
            '\r' => output.push_str("\\r"),
            '\t' => output.push_str("\\t"),
            ch if ch.is_control() => {
                output.push_str("\\u");
                output.push_str(&format!("{:04x}", ch as u32));
            }
            other => output.push(other),
        }
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn index_response_escapes_service_name() {
        let response = index_response("GemStone \"service\"");
        assert_eq!(response.status, 200);
        assert!(response.body.contains(r#"GemStone \"service\""#));
        assert!(response.body.contains(r#""gemstone":"/health/gemstone""#));
    }

    #[test]
    fn health_response_reports_success_and_errors_as_json() {
        assert_eq!(
            gemstone_health_response_from_result(Ok(7)),
            JsonResponse::ok(r#"{"result":7}"#)
        );

        let response =
            gemstone_health_response_from_result(Err(Error::MissingEnvironment("GS_PASSWORD")));
        assert_eq!(response.status, 500);
        assert!(response.body.contains("GS_PASSWORD"));
    }

    #[test]
    fn smallint_health_value_rejects_unexpected_values() {
        let err = smallint_health_value(Value::String("7".to_string())).unwrap_err();
        assert!(matches!(
            err,
            Error::UnexpectedType {
                expected: "SmallInt",
                ..
            }
        ));
    }

    #[test]
    fn json_escape_handles_quotes_and_control_characters() {
        assert_eq!(json_escape("a\"b\\c\n"), "a\\\"b\\\\c\\n");
        assert_eq!(json_escape("\u{0007}"), "\\u0007");
    }
}
