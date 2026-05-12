use proc_macro::{Delimiter, Group, Ident, Literal, Punct, Spacing, TokenStream, TokenTree};

#[proc_macro_derive(BridgeMapped, attributes(bridge))]
pub fn derive_bridge_mapped(input: TokenStream) -> TokenStream {
    match expand_bridge_mapped(input) {
        Ok(tokens) => tokens,
        Err(message) => compile_error(&message),
    }
}

fn expand_bridge_mapped(input: TokenStream) -> Result<TokenStream, String> {
    let mut tokens = input.into_iter();
    let mut struct_name = None;
    let mut body = None;

    while let Some(token) = tokens.next() {
        if matches_ident(&token, "struct") {
            struct_name = match tokens.next() {
                Some(TokenTree::Ident(ident)) => Some(ident.to_string()),
                _ => return Err("BridgeMapped can only derive for named structs".to_string()),
            };
            break;
        }
    }

    for token in tokens {
        if let TokenTree::Group(group) = token {
            if group.delimiter() == Delimiter::Brace {
                body = Some(group.stream());
                break;
            }
        }
    }

    let struct_name =
        struct_name.ok_or_else(|| "BridgeMapped can only derive for structs".to_string())?;
    let fields = parse_fields(
        body.ok_or_else(|| "BridgeMapped requires a struct with named fields".to_string())?,
    )?;
    if fields.is_empty() {
        return Err("BridgeMapped requires at least one named field".to_string());
    }

    let mut writes = String::new();
    let mut reads = String::new();
    for field in fields {
        let key = rust_string_literal(&field.key);
        let key_type = match field.key_type {
            KeyType::String => "gemstone_rs::BridgeKeyType::String",
            KeyType::Symbol => "gemstone_rs::BridgeKeyType::Symbol",
        };
        writes.push_str(&format!(
            "            (gemstone_rs::BridgeKey::new({key}, {key_type}), gemstone_rs::BridgeFieldWrite::to_bridge_field_value(&self.{name})),\n",
            name = field.name
        ));
        reads.push_str(&format!(
            "            {name}: gemstone_rs::BridgeFieldRead::read_bridge_field(dictionary, {key}, {key_type})?,\n",
            name = field.name
        ));
    }

    let source = format!(
        "impl gemstone_rs::BridgeMapped for {struct_name} {{
    fn to_bridge_value(&self) -> gemstone_rs::BridgeValue {{
        gemstone_rs::BridgeValue::keyed_dictionary([
{writes}        ])
    }}

    fn from_bridge_dictionary(dictionary: &mut gemstone_rs::BridgeDictionary<'_>) -> gemstone_rs::Result<Self> {{
        Ok(Self {{
{reads}        }})
    }}
}}"
    );
    source.parse().map_err(|err| format!("{err}"))
}

#[derive(Clone, Debug)]
struct Field {
    name: String,
    key: String,
    key_type: KeyType,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum KeyType {
    String,
    Symbol,
}

fn parse_fields(stream: TokenStream) -> Result<Vec<Field>, String> {
    split_fields(stream).into_iter().map(parse_field).collect()
}

fn split_fields(stream: TokenStream) -> Vec<Vec<TokenTree>> {
    let mut fields = Vec::new();
    let mut current = Vec::new();
    for token in stream {
        match &token {
            TokenTree::Punct(punct) if punct.as_char() == ',' => {
                if !current.is_empty() {
                    fields.push(std::mem::take(&mut current));
                }
            }
            _ => current.push(token),
        }
    }
    if !current.is_empty() {
        fields.push(current);
    }
    fields
}

fn parse_field(tokens: Vec<TokenTree>) -> Result<Field, String> {
    let field_text = tokens
        .iter()
        .map(TokenTree::to_string)
        .collect::<Vec<_>>()
        .join(" ");
    let mut key = None;
    let mut key_type = KeyType::String;
    let mut before_colon = Vec::new();
    let mut saw_colon = false;

    let mut index = 0;
    while index < tokens.len() {
        if is_bridge_attr(&tokens, index) {
            if let TokenTree::Group(group) = &tokens[index + 1] {
                parse_bridge_attr(group, &mut key, &mut key_type)?;
            }
            index += 2;
            continue;
        }

        match &tokens[index] {
            TokenTree::Punct(punct) if punct.as_char() == ':' => {
                saw_colon = true;
                break;
            }
            token => before_colon.push(token.clone()),
        }
        index += 1;
    }

    if key.is_none() {
        key = attr_value(&field_text, "key");
    }
    if let Some(value) = attr_value(&field_text, "key_type") {
        key_type = parse_key_type(&value)?;
    }

    if !saw_colon {
        return Err("BridgeMapped fields must be named fields".to_string());
    }

    let name = before_colon
        .iter()
        .rev()
        .find_map(|token| match token {
            TokenTree::Ident(ident) if ident.to_string() != "pub" => Some(ident.to_string()),
            _ => None,
        })
        .ok_or_else(|| "BridgeMapped could not find a field name".to_string())?;

    Ok(Field {
        key: key.unwrap_or_else(|| name.clone()),
        name,
        key_type,
    })
}

fn is_bridge_attr(tokens: &[TokenTree], index: usize) -> bool {
    matches!(tokens.get(index), Some(TokenTree::Punct(punct)) if punct.as_char() == '#')
        && matches!(tokens.get(index + 1), Some(TokenTree::Group(group)) if group.delimiter() == Delimiter::Bracket && group.to_string().trim_start().starts_with("bridge"))
}

fn parse_bridge_attr(
    group: &Group,
    key: &mut Option<String>,
    key_type: &mut KeyType,
) -> Result<(), String> {
    let text = group.to_string();
    if let Some(value) = attr_value(&text, "key") {
        *key = Some(value);
    }
    if let Some(value) = attr_value(&text, "key_type") {
        *key_type = parse_key_type(&value)?;
    }
    Ok(())
}

fn attr_value(text: &str, name: &str) -> Option<String> {
    let start = find_attr_name(text, name)?;
    let after_name = &text[start + name.len()..];
    let eq = after_name.find('=')?;
    let mut value = after_name[eq + 1..].trim_start();
    if let Some(rest) = value.strip_prefix('"') {
        let end = rest.find('"')?;
        return Some(rest[..end].to_string());
    }
    let end = value
        .find(|ch: char| ch == ',' || ch == ')' || ch.is_whitespace())
        .unwrap_or(value.len());
    value = &value[..end];
    Some(value.to_string())
}

fn find_attr_name(text: &str, name: &str) -> Option<usize> {
    let mut offset = 0;
    while let Some(found) = text[offset..].find(name) {
        let index = offset + found;
        let before_ok = index == 0
            || !text[..index]
                .chars()
                .next_back()
                .is_some_and(|ch| ch == '_' || ch.is_ascii_alphanumeric());
        let after_index = index + name.len();
        let after = &text[after_index..];
        let after_ok = after
            .chars()
            .next()
            .is_some_and(|ch| ch.is_whitespace() || ch == '=');
        if before_ok && after_ok {
            return Some(index);
        }
        offset = after_index;
    }
    None
}

fn parse_key_type(value: &str) -> Result<KeyType, String> {
    match value {
        "String" | "string" | "str" => Ok(KeyType::String),
        "Symbol" | "symbol" => Ok(KeyType::Symbol),
        other => Err(format!("unsupported bridge key_type: {other}")),
    }
}

fn matches_ident(token: &TokenTree, value: &str) -> bool {
    matches!(token, TokenTree::Ident(ident) if ident.to_string() == value)
}

fn rust_string_literal(value: &str) -> String {
    let mut literal = String::from("\"");
    for ch in value.chars() {
        match ch {
            '"' => literal.push_str("\\\""),
            '\\' => literal.push_str("\\\\"),
            '\n' => literal.push_str("\\n"),
            '\r' => literal.push_str("\\r"),
            '\t' => literal.push_str("\\t"),
            ch if ch.is_control() => literal.push_str(&format!("\\u{{{:x}}}", ch as u32)),
            ch => literal.push(ch),
        }
    }
    literal.push('"');
    literal
}

fn compile_error(message: &str) -> TokenStream {
    TokenStream::from_iter([
        TokenTree::Ident(Ident::new("compile_error", proc_macro::Span::call_site())),
        TokenTree::Punct(Punct::new('!', Spacing::Alone)),
        TokenTree::Group(Group::new(
            Delimiter::Parenthesis,
            TokenStream::from(TokenTree::Literal(Literal::string(message))),
        )),
        TokenTree::Punct(Punct::new(';', Spacing::Alone)),
    ])
}
