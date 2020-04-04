use crate::util::*;
use rusty_v8 as v8;
use serde::{de::DeserializeOwned, Serialize};
use serde_json::{Map, Value};
use std::convert::{TryFrom, TryInto};
use std::fmt::Debug;

pub trait FFICompat2<'sc, 'c>
where
    Self: Sized,
{
    type E: Debug;
    fn from_value(
        value: v8::Local<'sc, v8::Value>,
        scope: &mut impl v8::ToLocal<'sc>,
        _context: v8::Local<'c, v8::Context>,
    ) -> Result<Self, Self::E>;

    fn to_value(
        self,
        scope: &mut impl v8::ToLocal<'sc>,
        _context: v8::Local<'c, v8::Context>,
    ) -> Result<v8::Local<'sc, v8::Value>, Self::E>;
}

impl<'sc, 'c, T: Into<v8::Local<'sc, v8::Value>> + TryFrom<v8::Local<'sc, v8::Value>>>
    FFICompat2<'sc, 'c> for T
{
    type E = String;
    fn from_value(
        value: v8::Local<'sc, v8::Value>,
        _scope: &mut impl v8::ToLocal<'sc>,
        _context: v8::Local<'c, v8::Context>,
    ) -> Result<Self, String> {
        let value = value.try_into().ok();
        if value.is_none() {
            return Err("invalid type for argument in ffi call".to_string());
        }
        return Ok(value.unwrap());
    }

    fn to_value(
        self,
        _scope: &mut impl v8::ToLocal<'sc>,
        _context: v8::Local<'c, v8::Context>,
    ) -> Result<v8::Local<'sc, v8::Value>, String> {
        return Ok(self.into());
    }
}

pub trait FFICompat<'sc, 'c>
where
    Self: Sized,
{
    type E: Debug;
    fn from_value(
        value: v8::Local<'sc, v8::Value>,
        scope: &mut impl v8::ToLocal<'sc>,
        _context: v8::Local<'c, v8::Context>,
    ) -> Result<Self, Self::E>;

    fn to_value(
        self,
        scope: &mut impl v8::ToLocal<'sc>,
        context: v8::Local<'c, v8::Context>,
    ) -> Result<v8::Local<'sc, v8::Value>, Self::E>;
}

impl<'sc, 'c> FFICompat<'sc, 'c> for String {
    type E = String;
    fn from_value(
        value: v8::Local<'sc, v8::Value>,
        scope: &mut impl v8::ToLocal<'sc>,
        _context: v8::Local<'c, v8::Context>,
    ) -> Result<Self, String> {
        let value: Option<v8::Local<'sc, v8::String>> = value.try_into().ok();
        match value {
            Some(value) => Ok(value.to_rust_string_lossy(scope)),
            None => Err("invalid type for argument in ffi call, expected string".to_string()),
        }
    }

    fn to_value(
        self,
        scope: &mut impl v8::ToLocal<'sc>,
        _context: v8::Local<'c, v8::Context>,
    ) -> Result<v8::Local<'sc, v8::Value>, String> {
        return Ok(make_str(scope, &self));
    }
}

impl<'sc, 'c> FFICompat<'sc, 'c> for f64 {
    type E = String;
    fn from_value(
        value: v8::Local<'sc, v8::Value>,
        scope: &mut impl v8::ToLocal<'sc>,
        _context: v8::Local<'c, v8::Context>,
    ) -> Result<Self, String> {
        let value: Option<v8::Local<'sc, v8::Number>> = value.try_into().ok();
        match value.map(|n| n.number_value(scope)).flatten() {
            Some(value) => Ok(value),
            None => Err("invalid type for argument in ffi call, expected f64".to_string()),
        }
    }

    fn to_value(
        self,
        scope: &mut impl v8::ToLocal<'sc>,
        _context: v8::Local<'c, v8::Context>,
    ) -> Result<v8::Local<'sc, v8::Value>, String> {
        return Ok(make_num(scope, self));
    }
}

impl<'sc, 'c> FFICompat<'sc, 'c> for i64 {
    type E = String;
    fn from_value(
        value: v8::Local<'sc, v8::Value>,
        scope: &mut impl v8::ToLocal<'sc>,
        context: v8::Local<'c, v8::Context>,
    ) -> Result<Self, String> {
        f64::from_value(value, scope, context).map(|x| x as i64)
    }

    fn to_value(
        self,
        scope: &mut impl v8::ToLocal<'sc>,
        context: v8::Local<'c, v8::Context>,
    ) -> Result<v8::Local<'sc, v8::Value>, String> {
        return (self as f64).to_value(scope, context);
    }
}

impl<'sc, 'c> FFICompat<'sc, 'c> for u64 {
    type E = String;
    fn from_value(
        value: v8::Local<'sc, v8::Value>,
        scope: &mut impl v8::ToLocal<'sc>,
        context: v8::Local<'c, v8::Context>,
    ) -> Result<Self, String> {
        f64::from_value(value, scope, context).map(|x| x as u64)
    }

    fn to_value(
        self,
        scope: &mut impl v8::ToLocal<'sc>,
        context: v8::Local<'c, v8::Context>,
    ) -> Result<v8::Local<'sc, v8::Value>, String> {
        return (self as f64).to_value(scope, context);
    }
}

impl<'sc, 'c> FFICompat<'sc, 'c> for i32 {
    type E = String;
    fn from_value(
        value: v8::Local<'sc, v8::Value>,
        scope: &mut impl v8::ToLocal<'sc>,
        context: v8::Local<'c, v8::Context>,
    ) -> Result<Self, String> {
        f64::from_value(value, scope, context).map(|x| x as i32)
    }

    fn to_value(
        self,
        scope: &mut impl v8::ToLocal<'sc>,
        context: v8::Local<'c, v8::Context>,
    ) -> Result<v8::Local<'sc, v8::Value>, String> {
        return (self as f64).to_value(scope, context);
    }
}

impl<'sc, 'c> FFICompat<'sc, 'c> for u32 {
    type E = String;
    fn from_value(
        value: v8::Local<'sc, v8::Value>,
        scope: &mut impl v8::ToLocal<'sc>,
        context: v8::Local<'c, v8::Context>,
    ) -> Result<Self, String> {
        f64::from_value(value, scope, context).map(|x| x as u32)
    }

    fn to_value(
        self,
        scope: &mut impl v8::ToLocal<'sc>,
        context: v8::Local<'c, v8::Context>,
    ) -> Result<v8::Local<'sc, v8::Value>, String> {
        return (self as f64).to_value(scope, context);
    }
}

impl<'sc, 'c> FFICompat<'sc, 'c> for bool {
    type E = String;
    fn from_value(
        value: v8::Local<'sc, v8::Value>,
        _scope: &mut impl v8::ToLocal<'sc>,
        _context: v8::Local<'c, v8::Context>,
    ) -> Result<Self, String> {
        let value: Option<v8::Local<'sc, v8::Boolean>> = value.try_into().ok();
        match value.map(|n| n.is_true()) {
            Some(value) => Ok(value),
            None => Err("invalid type for argument in ffi call, expected boolean".to_string()),
        }
    }

    fn to_value(
        self,
        scope: &mut impl v8::ToLocal<'sc>,
        _context: v8::Local<'c, v8::Context>,
    ) -> Result<v8::Local<'sc, v8::Value>, String> {
        return Ok(make_bool(scope, self));
    }
}

impl<'sc, 'c> FFICompat<'sc, 'c> for () {
    type E = String;
    fn from_value(
        _value: v8::Local<'sc, v8::Value>,
        _scope: &mut impl v8::ToLocal<'sc>,
        _context: v8::Local<'c, v8::Context>,
    ) -> Result<Self, String> {
        Ok(())
    }

    fn to_value(
        self,
        scope: &mut impl v8::ToLocal<'sc>,
        _context: v8::Local<'c, v8::Context>,
    ) -> Result<v8::Local<'sc, v8::Value>, String> {
        return Ok(v8::undefined(scope).into());
    }
}

impl<'sc, 'c, T: FFICompat<'sc, 'c>> FFICompat<'sc, 'c> for Option<T> {
    type E = T::E;

    fn from_value(
        value: v8::Local<'sc, v8::Value>,
        scope: &mut impl v8::ToLocal<'sc>,
        context: v8::Local<'c, v8::Context>,
    ) -> Result<Self, Self::E> {
        Ok(T::from_value(value, scope, context).ok())
    }

    fn to_value(
        self,
        scope: &mut impl v8::ToLocal<'sc>,
        context: v8::Local<'c, v8::Context>,
    ) -> Result<v8::Local<'sc, v8::Value>, Self::E> {
        return Ok(self
            .map(|x| x.to_value(scope, context).ok())
            .flatten()
            .unwrap_or_else(|| v8::null(scope).into()));
    }
}

impl<'sc, 'c, E: Debug, T: FFICompat<'sc, 'c> + 'static> FFICompat<'sc, 'c> for Result<T, E> {
    type E = String;

    fn from_value(
        _value: v8::Local<'sc, v8::Value>,
        _scope: &mut impl v8::ToLocal<'sc>,
        _context: v8::Local<'c, v8::Context>,
    ) -> Result<Self, Self::E> {
        unimplemented!();
    }

    fn to_value(
        self,
        scope: &mut impl v8::ToLocal<'sc>,
        context: v8::Local<'c, v8::Context>,
    ) -> Result<v8::Local<'sc, v8::Value>, Self::E> {
        match self {
            Ok(v) => v.to_value(scope, context).map_err(|e| format!("{:?}", e)),
            Err(e) => Err(format!("{:?}", e)),
        }
    }
}

impl<'sc, 'c, T: FFICompat<'sc, 'c>> FFICompat<'sc, 'c> for Vec<T> {
    type E = T::E;

    fn from_value(
        value: v8::Local<'sc, v8::Value>,
        scope: &mut impl v8::ToLocal<'sc>,
        context: v8::Local<'c, v8::Context>,
    ) -> Result<Self, Self::E> {
        let value: Option<v8::Local<'sc, v8::Array>> = value.try_into().ok();
        let value = match value {
            Some(value) => value,
            None => {
                return Ok(vec![]);
            }
        };
        let mut values = vec![];
        for i in 0..value.length() {
            let local = value
                .get_index(scope, context, i)
                .unwrap_or_else(|| v8::undefined(scope).into());
            values.push(T::from_value(local, scope, context)?);
        }
        Ok(values)
    }

    fn to_value(
        self,
        scope: &mut impl v8::ToLocal<'sc>,
        context: v8::Local<'c, v8::Context>,
    ) -> Result<v8::Local<'sc, v8::Value>, Self::E> {
        let localled: Result<Vec<v8::Local<'sc, v8::Value>>, Self::E> = self
            .into_iter()
            .map(|x| x.to_value(scope, context))
            .collect();
        let localled = localled?;
        return Ok(v8::Array::new_with_elements(scope, &localled[..]).into());
    }
}

fn js_value_to_serde<'sc, 'c>(
    value: v8::Local<'sc, v8::Value>,
    scope: &mut impl v8::ToLocal<'sc>,
    context: v8::Local<'c, v8::Context>,
) -> Result<Value, String> {
    let nvalue: Result<v8::Local<v8::Array>, _> = value.try_into();
    if let Ok(nvalue) = nvalue {
        let mut values = vec![];
        for i in 0..nvalue.length() {
            let local = nvalue
                .get_index(scope, context, i)
                .unwrap_or_else(|| v8::undefined(scope).into());
            values.push(js_value_to_serde(local, scope, context)?);
        }
        return Ok(Value::Array(values));
    }
    let nvalue: Result<v8::Local<v8::Object>, _> = value.try_into();
    if let Ok(nvalue) = nvalue {
        let names = nvalue
            .get_own_property_names(scope, context)
            .unwrap_or(vec![]);
        let mut values: Map<String, Value> = Map::new();
        for name in names {
            let lname = make_str(scope, &name);
            let local = nvalue
                .get(scope, context, lname)
                .unwrap_or_else(|| v8::undefined(scope).into());
            values.insert(name, js_value_to_serde(local, scope, context)?);
        }
        return Ok(Value::Object(values));
    }
    let nvalue: Result<v8::Local<v8::String>, _> = value.try_into();
    if let Ok(nvalue) = nvalue {
        return Ok(Value::String(nvalue.to_rust_string_lossy(scope)));
    }
    let nvalue: Result<v8::Local<v8::Number>, _> = value.try_into();
    if let Ok(nvalue) = nvalue {
        return Ok(Value::Number(
            serde_json::Number::from_f64(nvalue.number_value(scope).unwrap_or(0.0)).unwrap(),
        ));
    }
    let nvalue: Result<v8::Local<v8::Boolean>, _> = value.try_into();
    if let Ok(nvalue) = nvalue {
        return Ok(Value::Bool(nvalue.is_true()));
    }
    if value.is_undefined() || value.is_null() {
        return Ok(Value::Null);
    }
    return Err("unknown js type for jsonification".to_string());
}

fn serde_to_js_value<'sc, 'c>(
    value: Value,
    scope: &mut impl v8::ToLocal<'sc>,
    context: v8::Local<'c, v8::Context>,
) -> Result<v8::Local<'sc, v8::Value>, String> {
    match value {
        Value::Array(array) => {
            let localled: Result<Vec<v8::Local<'sc, v8::Value>>, String> = array
                .into_iter()
                .map(|x| serde_to_js_value(x, scope, context))
                .collect();
            let localled = localled?;

            Ok(v8::Array::new_with_elements(scope, &localled[..]).into())
        }
        Value::Object(obj) => {
            let js_obj = v8::Object::new(scope);
            for (key, value) in obj.into_iter() {
                let key = make_str(scope, &key);
                js_obj.set(context, key, serde_to_js_value(value, scope, context)?);
            }
            Ok(js_obj.into())
        }
        Value::String(string) => Ok(make_str(scope, &string)),
        Value::Number(number) => Ok(make_num(scope, number.as_f64().unwrap_or(0.0))),
        Value::Bool(b) => Ok(make_bool(scope, b)),
        Value::Null => Ok(v8::null(scope).into()),
    }
}

/// marker trait for json mapping
pub trait FFIObject {}

impl<'sc, 'c, T: Serialize + DeserializeOwned + FFIObject + 'static> FFICompat<'sc, 'c> for T {
    type E = String;

    fn from_value(
        value: v8::Local<'sc, v8::Value>,
        scope: &mut impl v8::ToLocal<'sc>,
        context: v8::Local<'c, v8::Context>,
    ) -> Result<Self, String> {
        let value = js_value_to_serde(value, scope, context)?;
        serde_json::from_value(value).map_err(|e| format!("{:?}", e))
    }

    fn to_value(
        self,
        scope: &mut impl v8::ToLocal<'sc>,
        context: v8::Local<'c, v8::Context>,
    ) -> Result<v8::Local<'sc, v8::Value>, String> {
        let value = serde_json::to_value(self).map_err(|e| format!("{:?}", e))?;
        serde_to_js_value(value, scope, context)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusty_v8 as v8;
    use rusty_v8_helper_derive::v8_ffi;
    use serde::Deserialize;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::sync::Mutex;

    struct TestWrapper(String);

    #[derive(Serialize, Deserialize)]
    struct TestObj {
        value: String,
    }

    impl FFIObject for TestObj {}

    static TEST_RESPONSE: AtomicU64 = AtomicU64::new(0);

    #[v8_ffi]
    fn test_ffi_basic() {
        TEST_RESPONSE.store(1, Ordering::SeqCst);
    }

    #[v8_ffi]
    fn test_ffi_arg(arg: String) {
        if arg == "test1" {
            TEST_RESPONSE.store(2, Ordering::SeqCst);
        } else if arg == "test2" {
            TEST_RESPONSE.store(3, Ordering::SeqCst);
        }
    }

    #[v8_ffi]
    fn test_ffi_opt_arg(arg: Option<String>) {
        match arg {
            None => {
                TEST_RESPONSE.store(4, Ordering::SeqCst);
            }
            Some(_) => {
                TEST_RESPONSE.store(5, Ordering::SeqCst);
            }
        }
    }

    #[v8_ffi]
    fn test_ffi_return() -> String {
        "test".to_string()
    }

    #[v8_ffi]
    fn test_ffi_roundtrip(arg: String) -> String {
        return arg;
    }

    #[v8_ffi]
    fn test_ffi_roundtrip_check(arg: String) {
        if arg == "test" {
            TEST_RESPONSE.store(6, Ordering::SeqCst);
        }
    }

    #[v8_ffi]
    fn test_ffi_result(arg: String) -> Result<String, String> {
        if arg == "success" {
            return Ok(arg);
        } else {
            return Err(arg);
        }
    }

    #[v8_ffi]
    fn test_ffi_vec(arg: Vec<String>) -> Vec<String> {
        if arg.len() == 1 {
            TEST_RESPONSE.store(7, Ordering::SeqCst);
            return vec!["test1".to_string(), "test2".to_string()];
        } else if arg.len() == 2 {
            TEST_RESPONSE.store(8, Ordering::SeqCst);
            return vec![];
        } else {
            return vec![];
        }
    }

    #[v8_ffi]
    fn test_ffi_wrap(this: &TestWrapper) {
        if this.0 == "test1" {
            TEST_RESPONSE.store(9, Ordering::SeqCst);
        } else if this.0 == "test2" {
            TEST_RESPONSE.store(10, Ordering::SeqCst);
        }
    }

    #[v8_ffi]
    fn test_ffi_wrap_mut(this: &mut TestWrapper) {
        if this.0 == "test1" {
            this.0 = "test3".to_string();
        } else if this.0 == "test3" {
            this.0 = "test4".to_string();
        }
    }

    #[v8_ffi]
    fn test_ffi_obj(arg: TestObj) -> TestObj {
        if arg.value == "test1" {
            TEST_RESPONSE.store(11, Ordering::SeqCst);
            return TestObj {
                value: "test2".to_string(),
            };
        } else if arg.value == "test2" {
            TEST_RESPONSE.store(12, Ordering::SeqCst);
            return arg;
        } else {
            return arg;
        }
    }

    #[v8_ffi]
    fn test_ffi_result_join(arg: String) -> Result<String, Box<dyn std::error::Error>> {
        TEST_RESPONSE.store(13, Ordering::SeqCst);
        Ok(arg)
    }

    #[v8_ffi]
    fn test_ffi_unit() -> () {
        TEST_RESPONSE.store(14, Ordering::SeqCst);
    }

    #[test]
    fn exec_tests() {
        let platform = v8::new_default_platform();
        v8::V8::initialize_platform(platform);
        v8::V8::initialize();
        let mut create_params = v8::Isolate::create_params();
        create_params.set_array_buffer_allocator(v8::new_default_allocator());
        let mut isolate = v8::Isolate::new(create_params);
        let mut hs = v8::HandleScope::new(&mut isolate);
        let scope = hs.enter();
        let context = v8::Context::new(scope);
        let mut cs = v8::ContextScope::new(scope, context);
        let scope = cs.enter();
        let global = context.global(scope);
        //basic
        global.set(
            context,
            make_str(scope, "test_ffi_basic"),
            load_v8_ffi!(test_ffi_basic, scope, context),
        );
        run_script(scope, context, "test_ffi_basic()");
        assert_eq!(TEST_RESPONSE.load(Ordering::SeqCst), 1);
        //arg
        global.set(
            context,
            make_str(scope, "test_ffi_arg"),
            load_v8_ffi!(test_ffi_arg, scope, context),
        );
        run_script(scope, context, "test_ffi_arg('test1')");
        assert_eq!(TEST_RESPONSE.load(Ordering::SeqCst), 2);
        run_script(scope, context, "test_ffi_arg('test2')");
        assert_eq!(TEST_RESPONSE.load(Ordering::SeqCst), 3);
        // optional arg
        global.set(
            context,
            make_str(scope, "test_ffi_opt_arg"),
            load_v8_ffi!(test_ffi_opt_arg, scope, context),
        );
        run_script(scope, context, "test_ffi_opt_arg(null)");
        assert_eq!(TEST_RESPONSE.load(Ordering::SeqCst), 4);
        run_script(scope, context, "test_ffi_opt_arg('test')");
        assert_eq!(TEST_RESPONSE.load(Ordering::SeqCst), 5);
        run_script(scope, context, "test_ffi_opt_arg(undefined)");
        assert_eq!(TEST_RESPONSE.load(Ordering::SeqCst), 4);
        run_script(scope, context, "test_ffi_opt_arg('test')");
        assert_eq!(TEST_RESPONSE.load(Ordering::SeqCst), 5);
        run_script(scope, context, "test_ffi_opt_arg(77)");
        assert_eq!(TEST_RESPONSE.load(Ordering::SeqCst), 4);
        run_script(scope, context, "test_ffi_opt_arg('test')");
        assert_eq!(TEST_RESPONSE.load(Ordering::SeqCst), 5);
        // return val & roundtripping
        global.set(
            context,
            make_str(scope, "test_ffi_return"),
            load_v8_ffi!(test_ffi_return, scope, context),
        );
        global.set(
            context,
            make_str(scope, "test_ffi_roundtrip"),
            load_v8_ffi!(test_ffi_roundtrip, scope, context),
        );
        global.set(
            context,
            make_str(scope, "test_ffi_roundtrip_check"),
            load_v8_ffi!(test_ffi_roundtrip_check, scope, context),
        );
        run_script(
            scope,
            context,
            "test_ffi_roundtrip_check(test_ffi_roundtrip(test_ffi_return()))",
        );
        assert_eq!(TEST_RESPONSE.load(Ordering::SeqCst), 6);
        // throw on bad type
        run_script(
            scope,
            context,
            "try { test_ffi_arg(undefined) } catch (e) { test_ffi_arg('test2') }",
        );
        assert_eq!(TEST_RESPONSE.load(Ordering::SeqCst), 3);
        // can pass result
        global.set(
            context,
            make_str(scope, "test_ffi_result"),
            load_v8_ffi!(test_ffi_result, scope, context),
        );
        run_script(
            scope,
            context,
            "try { test_ffi_result('success') } catch (e) { test_ffi_arg('test1') }",
        );
        assert_eq!(TEST_RESPONSE.load(Ordering::SeqCst), 3);
        // can fail result
        run_script(
            scope,
            context,
            "try { test_ffi_result('failure') } catch (e) { test_ffi_arg('test1') }",
        );
        assert_eq!(TEST_RESPONSE.load(Ordering::SeqCst), 2);
        // vectors
        global.set(
            context,
            make_str(scope, "test_ffi_vec"),
            load_v8_ffi!(test_ffi_vec, scope, context),
        );
        run_script(scope, context, "test_ffi_vec(['test'])");
        assert_eq!(TEST_RESPONSE.load(Ordering::SeqCst), 7);
        run_script(scope, context, "test_ffi_vec(test_ffi_vec(['test']))");
        assert_eq!(TEST_RESPONSE.load(Ordering::SeqCst), 8);
        // immutable ffi wrap
        global.set(
            context,
            make_str(scope, "test_ffi_wrap"),
            load_v8_ffi!(test_ffi_wrap, scope, context),
        );
        let test_ffi_wrap_data = make_object_wrap(scope, context, TestWrapper("test1".to_string()));
        global.set(
            context,
            make_str(scope, "test_ffi_wrap_data"),
            test_ffi_wrap_data.get(scope).unwrap().into(),
        );
        run_script(scope, context, "test_ffi_wrap.bind(test_ffi_wrap_data)()");
        assert_eq!(TEST_RESPONSE.load(Ordering::SeqCst), 9);
        let test_ffi_wrap_data2 =
            make_object_wrap(scope, context, TestWrapper("test2".to_string()));
        global.set(
            context,
            make_str(scope, "test_ffi_wrap_data2"),
            test_ffi_wrap_data2.get(scope).unwrap().into(),
        );
        run_script(scope, context, "test_ffi_wrap.bind(test_ffi_wrap_data2)()");
        assert_eq!(TEST_RESPONSE.load(Ordering::SeqCst), 10);
        global.set(
            context,
            make_str(scope, "test_ffi_wrap_mut"),
            load_v8_ffi!(test_ffi_wrap_mut, scope, context),
        );
        let test_ffi_wrap_mut_data =
            make_object_wrap(scope, context, Mutex::new(TestWrapper("test1".to_string())));
        global.set(
            context,
            make_str(scope, "test_ffi_wrap_mut_data"),
            test_ffi_wrap_mut_data.get(scope).unwrap().into(),
        );
        run_script(
            scope,
            context,
            "test_ffi_wrap_mut.bind(test_ffi_wrap_mut_data)()",
        );
        assert_eq!(
            test_ffi_wrap_mut_data
                .unwrap(scope)
                .unwrap()
                .lock()
                .unwrap()
                .0,
            "test3"
        );
        run_script(
            scope,
            context,
            "test_ffi_wrap_mut.bind(test_ffi_wrap_mut_data)()",
        );
        assert_eq!(
            test_ffi_wrap_mut_data
                .unwrap(scope)
                .unwrap()
                .lock()
                .unwrap()
                .0,
            "test4"
        );
        global.set(
            context,
            make_str(scope, "test_ffi_obj"),
            load_v8_ffi!(test_ffi_obj, scope, context),
        );
        run_script(scope, context, "test_ffi_obj({ value: 'test1' })");
        assert_eq!(TEST_RESPONSE.load(Ordering::SeqCst), 11);
        run_script(
            scope,
            context,
            "test_ffi_obj(test_ffi_obj({ value: 'test1' }))",
        );
        assert_eq!(TEST_RESPONSE.load(Ordering::SeqCst), 12);
        global.set(
            context,
            make_str(scope, "test_ffi_result_join"),
            load_v8_ffi!(test_ffi_result_join, scope, context),
        );
        run_script(scope, context, "test_ffi_result_join('test')");
        assert_eq!(TEST_RESPONSE.load(Ordering::SeqCst), 13);
        global.set(
            context,
            make_str(scope, "test_ffi_unit"),
            load_v8_ffi!(test_ffi_unit, scope, context),
        );
        run_script(scope, context, "test_ffi_unit()");
        assert_eq!(TEST_RESPONSE.load(Ordering::SeqCst), 14);
    }
}
