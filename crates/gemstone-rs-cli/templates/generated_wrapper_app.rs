// Requires a live GemStone/S stone.
//
// Expected output includes:
//
// generated wrapper printString: 7

mod gemstone_wrappers {
    use gemstone_rs::{Error, Oop, Result, Session, Value};

    pub struct Object<'a> {
        session: &'a mut Session,
        oop: Oop,
    }

    impl<'a> Object<'a> {
        pub fn from_oop(session: &'a mut Session, oop: Oop) -> Self {
            Self { session, oop }
        }

        /// Return the receiver printString.
        pub fn print_string(&mut self) -> Result<String> {
            let value = self.session.perform(self.oop, "printString", &[])?;
            match value {
                Value::String(value) => Ok(value),
                Value::Oop(oop) => self.session.fetch_string(oop),
                other => Err(Error::UnexpectedType {
                    expected: "String",
                    actual: format!("{other:?}"),
                }),
            }
        }
    }
}

use gemstone_rs::{Config, Session};

fn main() -> gemstone_rs::Result<()> {
    let mut session = Session::login(Config::from_env()?)?;
    let oop = session.smallint_oop(7);
    let mut object = gemstone_wrappers::Object::from_oop(&mut session, oop);

    let printed = object.print_string()?;
    assert_eq!(printed, "7");
    println!("generated wrapper printString: {printed}");

    Ok(())
}
