use std::collections::HashMap;
use std::iter::{Chain, FilterMap, Map};
use std::str::FromStr;
use either::Either;
use enum_map::{EnumArray, EnumMap, Iter};
use evalexpr::{Context, ContextWithMutableFunctions, ContextWithMutableVariables, EvalexprError, EvalexprNumericTypes, EvalexprResult, Function, IterateVariablesContext, Value, ValueType};
use evalexpr::error::EvalexprResultValue;

type EnumContextValue<NumericTypes> = Option<Either<Value<NumericTypes>, Function<NumericTypes>>>;

pub struct EnumContext<E: EnumArray<EnumContextValue<NumericTypes>>, NumericTypes: EvalexprNumericTypes> {
    variables_and_functions: EnumMap<E, EnumContextValue<NumericTypes>>,
    variables_fallback: HashMap<String, Value<NumericTypes>>,
    functions_fallback: HashMap<String, Function<NumericTypes>>,
    without_builtin_functions: bool,
}


impl<E: EnumArray<EnumContextValue<NumericTypes>>, NumericTypes: EvalexprNumericTypes> EnumContext<E, NumericTypes> where E: Clone {
    fn get_value_enum(&self, identifier: E) -> Option<&Value<NumericTypes>> {
        match self.variables_and_functions[identifier].as_ref() {
            Some(Either::Left(x)) => {
                Some(x)
            }
            _ => {
                None
            }
        }
    }

    fn get_function_enum(&self, identifier: E) -> Option<&Function<NumericTypes>> {
        match self.variables_and_functions[identifier].as_ref() {
            Some(Either::Right(x)) => {
                Some(x)
            }
            _ => None
        }
    }

    fn set_value_enum(&mut self, identifier: E, value: Value<NumericTypes>) -> EvalexprResult<(), NumericTypes> {
        match &mut self.variables_and_functions[identifier] {
            Some(found) => {
                match found {
                    Either::Left(existing_value) => {
                        if ValueType::from(&existing_value) == ValueType::from(&value) {
                            *existing_value = value;
                            Ok(())
                        } else {
                            Err(EvalexprError::expected_type(existing_value, value))
                        }
                    }
                    Either::Right(_) => {
                        Err(EvalexprError::CustomMessage("Can not replace a function with a value!".to_string()))
                    }
                }
            }
            replace => {
                *replace = Some(Either::Left(value));
                Ok(())
            }
        }
    }

    fn set_function_enum(&mut self, identifier: E, function: Function<NumericTypes>) -> EvalexprResult<(), NumericTypes> {
        match &mut self.variables_and_functions[identifier] {
            Some(found) => {
                match found {
                    Either::Left(_) => {
                        Err(EvalexprError::CustomMessage("Can not replace a value with a function!".to_string()))
                    }
                    Either::Right(existing_function) => {
                        *existing_function = function;
                        Ok(())
                    }
                }
            }
            set => {
                *set = Some(Either::Right(function));
                Ok(())
            }
        }
    }
}

impl<E: EnumArray<EnumContextValue<NumericTypes>>, NumericTypes: EvalexprNumericTypes> Context for EnumContext<E, NumericTypes> where E: FromStr + Clone {
    type NumericTypes = NumericTypes;

    fn get_value(&self, identifier: &str) -> Option<&Value<NumericTypes>> {
        match identifier.parse::<E>() {
            Ok(value) => {
                self.get_value_enum(value)
            }
            Err(_) => {
                self.variables_fallback.get(identifier)
            }
        }
    }

    fn call_function(&self, identifier: &str, argument: &Value<NumericTypes>) -> EvalexprResultValue<NumericTypes> {
        match identifier.parse::<E>() {
            Ok(value) => {
                match self.get_function_enum(value) {
                    None => {}
                    Some(fkt) => {
                        return fkt.call(argument)
                    }
                }
            }
            Err(_) => {}
        }
        match self.functions_fallback.get(identifier) {
            Some(fkt) => {
                fkt.call(argument)
            }
            None => {
                Err(EvalexprError::FunctionIdentifierNotFound(identifier.to_string()))
            }
        }
    }

    fn are_builtin_functions_disabled(&self) -> bool {
        self.without_builtin_functions
    }

    fn set_builtin_functions_disabled(&mut self, disabled: bool) -> EvalexprResult<(), NumericTypes> {
        self.without_builtin_functions = disabled;
        Ok(())
    }
}

impl<E: EnumArray<EnumContextValue<NumericTypes>>, NumericTypes: EvalexprNumericTypes> ContextWithMutableVariables for EnumContext<E, NumericTypes> where E: FromStr + Clone {
    fn set_value(&mut self, identifier: String, value: Value<NumericTypes>) -> EvalexprResult<(), NumericTypes> {
        match identifier.parse::<E>() {
            Ok(identifier) => {
                self.set_value_enum(identifier, value)
            }
            Err(_) => {
                if let Some(existing_value) = self.variables_fallback.get_mut(&identifier) {
                    if ValueType::from(&existing_value) == ValueType::from(&value) {
                        *existing_value = value;
                    } else {
                        return Err(EvalexprError::expected_type(existing_value, value))
                    }
                } else {
                    self.variables_fallback.insert(identifier, value);
                }
                Ok(())
            }
        }
    }
}

impl<E: EnumArray<EnumContextValue<NumericTypes>>, NumericTypes: EvalexprNumericTypes> ContextWithMutableFunctions for EnumContext<E, NumericTypes> where E: FromStr + Clone {
    fn set_function(&mut self, identifier: String, function: Function<NumericTypes>) -> EvalexprResult<(), NumericTypes> {
        match identifier.parse::<E>() {
            Ok(identifier) => {
                self.set_function_enum(identifier, function)
            }
            Err(_) => {
                self.functions_fallback.insert(identifier, function);
                Ok(())
            }
        }
    }
}

impl<E: EnumArray<EnumContextValue<NumericTypes>>, NumericTypes: EvalexprNumericTypes> IterateVariablesContext for EnumContext<E, NumericTypes> where E: FromStr + Clone + ToString {
    type VariableIterator<'a> = Chain<
        FilterMap<
            Iter<'a, E, EnumContextValue<NumericTypes>>,
            fn((E, &'a EnumContextValue<NumericTypes>)) -> Option<(String, Value<NumericTypes>)>
        >,
        Map<
            std::collections::hash_map::Iter<'a, String, Value<NumericTypes>>,
            fn((&'a String, &'a Value<NumericTypes>)) -> (String, Value<NumericTypes>),
        >
    > where Self: 'a;
    type VariableNameIterator<'a> = Chain<
        FilterMap<
            Iter<'a, E, EnumContextValue<NumericTypes>>,
            fn((E, &'a EnumContextValue<NumericTypes>)) -> Option<String>
        >,
        std::iter::Cloned<std::collections::hash_map::Keys<'a, String, Value<NumericTypes>>>
    > where Self: 'a;

    fn iter_variables(&self) -> Self::VariableIterator<'_> {

        let a: FilterMap<
            Iter<E, EnumContextValue<NumericTypes>>,
            fn((E, &EnumContextValue<NumericTypes>)) -> Option<(String, Value<NumericTypes>)>
        > = self.variables_and_functions.iter().filter_map(|(identifier, value)| {
            match value {
                Some(Either::Left(value)) => {
                    Some((identifier.to_string(), value.clone()))
                }
                _ => None,
            }
        });

        let b: Map<
            std::collections::hash_map::Iter<String, Value<NumericTypes>>,
            fn((&String, &Value<NumericTypes>)) -> (String, Value<NumericTypes>),
        > = self.variables_fallback
            .iter()
            .map(|(string, value)| (string.clone(), value.clone()));

        a.chain(b)
    }

    fn iter_variable_names(&self) -> Self::VariableNameIterator<'_> {
        let a: FilterMap<
            Iter<E, EnumContextValue<NumericTypes>>,
            fn((E, &EnumContextValue<NumericTypes>)) -> Option<String>
        > = self.variables_and_functions.iter().filter_map(|(identifier, value)| {
            match value {
                Some(Either::Left(_)) => {
                    Some(identifier.to_string())
                }
                _ => None,
            }
        });

        let b: std::iter::Cloned<std::collections::hash_map::Keys<String, Value<NumericTypes>>> =
            self.variables_fallback.keys().cloned();

        a.chain(b)
    }
}

impl<E: EnumArray<EnumContextValue<NumericTypes>>, NumericTypes: EvalexprNumericTypes> Default for EnumContext<E, NumericTypes> {
    fn default() -> Self {
        Self {
            variables_and_functions: EnumMap::from_fn(|_| None),
            variables_fallback: HashMap::new(),
            functions_fallback: HashMap::new(),
            without_builtin_functions: false,
        }
    }
}