use crate::Type::function;
use super::some::types::Assosiated::function;
use super::Type::function;
use Type::function;
use crate::{Type::function, other::Type::function};
use does_not_work_for::this::function;

use some::{foreign::Type, OtherType::function, some::CONSTANT, Type::CONSTANT};

// tag: boolean
true;
false;
// tag(module): module
module::CONSTANT_VALUE;
module::Type::function;
// tag: variable
{asdklfjsalkfjsdf}
// tag(some_field): property
struct Mytype {some_field: String}
// tag: type.special
Self
{Mytype.some_field}
// tag: type.variant
module::Enum::Variant;
// tag: function.argument
function(argument)
// tag: variant.special
Err(), None;
// tag: variant.special.success
Ok(), Some();
enum MyEnum {
    //tag: variant.definition
    Ok(),
    Some(),
    Err(),
    None,
}


other::Enum::NestedJibberish(MyEnum::Ok(MyEnum::Some(MyEnum::Err(MyEnum::None))));
macro!(macro_argument, [MyType])
//tag(derive): attribute.special
//tag(trait): type.interface
#[derive(Trait)]
//tag(attribute: attribute)
//tag(attribute_argument: attribute.argument)
#[attribute(attribute_argument, Trait)]
//tag: label
'label:
funciton_that_takes_a_function_and_a_value(some::scoped::function, value)
function_that_takes_a_function_and_a_value(some::MyType::function, value)
funciton_that_takes_a_function_and_a_value(does_not_work_without_scope, value)

//tag: keyword
SomeType as AnotherType;
async const default; dyn enum extern fn; let; macro_rules!; mod;
move pub ref static struct trait type union unsafe for use where impl crate; mut super

//tag: keyword.control
loop while in continue break return if {} else; match; await yield
{
for _ in .. {}
}

#[module::macro]
mod my_module {
use super::*;

    #[cfg(debug_assertions)]
    enum MyEnum {
        None,
        WithValue(String),
        WithFields{
            field_one: u8,
            field_two: u8,
        }
    }

    #[derive(Debug)]
    struct MyType {
        some_field: String,
    }

    impl Trait for Mytype{}

    impl<'a, T, U> GenericTrait<u8> for Mytype
        where
            T: FromIterator<String>,
            U: FnOnce(String) -> impl Display,
    {
        pub fn do_something(&self) -> Self {
            Self {
                some_field: self.some_field.clone()
            }
        }

        pub fn so_something_else(self) -> Result<MyEnum, String> {
            let result:MyEnum = MyEnum::WithFields {
                field_one: some::path::give_me_a_u8(),
                field_two: pass_in_a_function(some::function)
            };
            if 3 == 8 {return} else {}

            if let Some(value) = Some(8) {
                assert!(value == 8)
            } else {
                return
            }

            'main_loop: for index in 0..10 {
                while true {
                    match index {
                        index @ 3..8 => break 'main_loop,
                        _ => continue 'main_loop,
                    }
                }
            }
            let _ = Ok(Some(Err(None)));


            return Ok(result)

        }

    }

}


trait MyTrait<T> where T: Display {

    fn print_me(&self) {
        println!("{}", self)
    }
}
