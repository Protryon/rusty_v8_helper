use crate::ObjectWrap;
use rusty_v8 as v8;
use std::rc::Rc;

pub fn make_str<'sc>(scope: &mut impl v8::ToLocal<'sc>, value: &str) -> v8::Local<'sc, v8::Value> {
    v8::String::new(scope, value).unwrap().into()
}

pub fn make_num<'sc>(scope: &mut impl v8::ToLocal<'sc>, value: f64) -> v8::Local<'sc, v8::Value> {
    v8::Number::new(scope, value).into()
}

pub fn make_bool<'sc>(scope: &mut impl v8::ToLocal<'sc>, value: bool) -> v8::Local<'sc, v8::Value> {
    v8::Boolean::new(scope, value).into()
}

pub fn throw_exception<'sc>(scope: &mut impl v8::ToLocal<'sc>, message: &str) {
    let message = make_str(scope, message);
    scope.isolate().throw_exception(message);
}

pub fn run_script<'sc>(
    scope: &mut impl v8::ToLocal<'sc>,
    context: v8::Local<v8::Context>,
    script: &str,
) -> Option<v8::Local<'sc, v8::Value>> {
    let script = make_str(scope, script);
    let script = script.to_string(scope).unwrap();
    let mut compiled = v8::Script::compile(scope, context, script, None);
    compiled.as_mut().map(|x| x.run(scope, context)).flatten()
}

pub fn make_object_wrap<'sc, T>(
    scope: &mut impl v8::ToLocal<'sc>,
    context: v8::Local<v8::Context>,
    wrap: T,
) -> ObjectWrap<T> {
    let mut obj = v8::ObjectTemplate::new(scope);
    obj.set_internal_field_count(2);
    let obj = obj.new_instance(scope, context).unwrap();
    ObjectWrap::new(scope, obj, wrap)
}

pub fn make_object_wrap_rc<'sc, T>(
    scope: &mut impl v8::ToLocal<'sc>,
    context: v8::Local<v8::Context>,
    wrap: Rc<T>,
) -> ObjectWrap<T> {
    let mut obj = v8::ObjectTemplate::new(scope);
    obj.set_internal_field_count(2);
    let obj = obj.new_instance(scope, context).unwrap();
    ObjectWrap::new_rc(scope, obj, wrap)
}
