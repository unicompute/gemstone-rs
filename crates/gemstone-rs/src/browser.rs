use crate::{Result, Session};

pub const ALL_PROTOCOLS: &str = "-- all --";

pub struct Browser<'a> {
    session: &'a mut Session,
}

impl<'a> Browser<'a> {
    pub fn new(session: &'a mut Session) -> Self {
        Self { session }
    }

    pub fn dictionaries(&mut self) -> Result<Vec<String>> {
        eval_lines(self.session, dictionaries_source())
    }

    pub fn classes(&mut self, dictionary: &str) -> Result<Vec<String>> {
        eval_lines(self.session, &classes_source(dictionary))
    }

    /// Return protocols for a class. Pass an empty dictionary to resolve the
    /// class through the active user's symbol list.
    pub fn protocols(
        &mut self,
        class_name: &str,
        meta: bool,
        dictionary: &str,
    ) -> Result<Vec<String>> {
        eval_lines(
            self.session,
            &protocols_source(class_name, meta, dictionary),
        )
    }

    /// Return selectors for a class/protocol. Pass an empty dictionary to
    /// resolve the class through the active user's symbol list.
    pub fn methods(
        &mut self,
        class_name: &str,
        protocol: &str,
        meta: bool,
        dictionary: &str,
    ) -> Result<Vec<String>> {
        eval_lines(
            self.session,
            &methods_source(class_name, meta, dictionary, protocol),
        )
    }

    /// Return class definition text or a compiled method source string. Pass
    /// an empty dictionary to resolve the class through the active user's
    /// symbol list.
    pub fn source(
        &mut self,
        class_name: &str,
        selector: &str,
        meta: bool,
        dictionary: &str,
    ) -> Result<String> {
        eval_string(
            self.session,
            &source_source(class_name, meta, dictionary, selector),
        )
    }
}

pub fn dictionaries_source() -> &'static str {
    "[ | stream |\n\
     stream := WriteStream on: String new.\n\
     System myUserProfile symbolList do: [:dict |\n\
       stream nextPutAll: (([dict name] on: Error do: [:e | dict printString]) asString); lf\n\
     ].\n\
     stream contents\n\
     ] value"
}

pub fn classes_source(dictionary: &str) -> String {
    format!(
        "[ | dict stream classNames |\n\
         dict := {}.\n\
         stream := WriteStream on: String new.\n\
         dict ifNil: [ '' ] ifNotNil: [\n\
           classNames := dict keys select: [:each | (dict at: each) isBehavior].\n\
           classNames asSortedCollection do: [:cls | stream nextPutAll: cls asString; lf].\n\
           stream contents\n\
         ]\n\
         ] value",
        dict_expr(dictionary)
    )
}

pub fn protocols_source(class_name: &str, meta: bool, dictionary: &str) -> String {
    format!(
        "[ | cls stream |\n\
         cls := {}.\n\
         stream := WriteStream on: String new.\n\
         cls ifNil: [ '' ] ifNotNil: [\n\
           cls categoryNames asSortedCollection do: [:cat | stream nextPutAll: cat asString; lf].\n\
           stream contents\n\
         ]\n\
         ] value",
        behavior_expr(class_name, meta, dictionary)
    )
}

pub fn methods_source(class_name: &str, meta: bool, dictionary: &str, protocol: &str) -> String {
    format!(
        "[ | cls stream sels |\n\
         cls := {}.\n\
         stream := WriteStream on: String new.\n\
         cls ifNil: [ '' ] ifNotNil: [\n\
           sels := '{}' = '{}'\n\
             ifTrue: [ cls selectors ]\n\
             ifFalse: [ cls selectorsIn: '{}' asSymbol ].\n\
           sels ifNil: [ sels := #() ].\n\
           sels asSortedCollection do: [:sel | stream nextPutAll: sel asString; lf].\n\
           stream contents\n\
         ]\n\
         ] value",
        behavior_expr(class_name, meta, dictionary),
        st_escape(protocol),
        ALL_PROTOCOLS,
        st_escape(protocol)
    )
}

pub fn source_source(class_name: &str, meta: bool, dictionary: &str, selector: &str) -> String {
    format!(
        "| cls meth |\n\
         cls := {}.\n\
         cls ifNil: [ '' ] ifNotNil: [\n\
           '{}' isEmpty\n\
             ifTrue: [\n\
               (cls respondsTo: #definition)\n\
                 ifTrue: [[cls definition asString] on: Error do: [:e | cls printString]]\n\
                 ifFalse: [cls printString]\n\
             ]\n\
             ifFalse: [\n\
               meth := cls compiledMethodAt: '{}' asSymbol ifAbsent: [nil].\n\
               meth ifNil: [ '' ] ifNotNil: [[meth sourceString] on: Error do: [:e | '']]\n\
             ]\n\
         ]",
        behavior_expr(class_name, meta, dictionary),
        st_escape(selector),
        st_escape(selector)
    )
}

pub fn dict_expr(dictionary: &str) -> String {
    format!(
        "(System myUserProfile symbolList objectNamed: '{}' asSymbol)",
        st_escape(dictionary)
    )
}

pub fn class_expr(class_name: &str, dictionary: &str) -> String {
    let escaped_name = st_escape(class_name);
    if dictionary.trim().is_empty() {
        format!(
            "(System myUserProfile symbolList objectNamed: '{}' asSymbol)",
            escaped_name
        )
    } else {
        format!(
            "({} at: '{}' asSymbol ifAbsent: [nil])",
            dict_expr(dictionary),
            escaped_name
        )
    }
}

pub fn behavior_expr(class_name: &str, meta: bool, dictionary: &str) -> String {
    let expr = class_expr(class_name, dictionary);
    if meta {
        format!("({expr} ifNil: [nil] ifNotNil: [:cls | cls class])")
    } else {
        expr
    }
}

pub fn st_escape(value: &str) -> String {
    value.replace('\'', "''")
}

fn eval_lines(session: &mut Session, source: &str) -> Result<Vec<String>> {
    Ok(eval_string(session, source)?
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(str::to_string)
        .collect())
}

fn eval_string(session: &mut Session, source: &str) -> Result<String> {
    let oop = session.execute(source)?;
    session.fetch_string(oop)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smalltalk_literals_escape_quotes() {
        assert_eq!(st_escape("User'sGlobals"), "User''sGlobals");
        assert!(class_expr("Example'sClass", "UserGlobals").contains("Example''sClass"));
    }

    #[test]
    fn behavior_expression_targets_metaclass_when_requested() {
        assert!(
            behavior_expr("Object", true, "UserGlobals").contains("ifNotNil: [:cls | cls class]")
        );
    }

    #[test]
    fn method_source_checks_all_protocol_marker() {
        let source = methods_source("Object", false, "UserGlobals", ALL_PROTOCOLS);
        assert!(source.contains("'-- all --' = '-- all --'"));
        assert!(source.contains("cls selectors"));
    }

    #[test]
    fn source_source_fetches_selector_source() {
        let source = source_source("Object", false, "UserGlobals", "printString");
        assert!(source.contains("compiledMethodAt: 'printString' asSymbol"));
    }
}
