use std::iter::Chain;
use evalexpr::{Context, ContextWithMutableFunctions, ContextWithMutableVariables, EvalexprError, EvalexprResult, Function, IterateVariablesContext, Value};


/// Conbines a global and a local context in a meaningful way.
#[derive(Debug)]
pub struct CombinedContextWrapper<'a, A: ?Sized, B: ?Sized> {
    local_context: &'a mut A,
    global_context: &'a B
}

impl<'a, A, B> CombinedContextWrapper<'a, A, B> {
    #[inline(always)]
    pub fn new(local_context: &'a mut A, global_context: &'a B) -> Self {
        Self { local_context, global_context }
    }
}

pub trait CombineableContext<B> where B: Context {
    fn combine_with<'a>(self: &'a mut Self, other: &'a B) -> CombinedContextWrapper<Self, B>;
}

impl<A, B> CombineableContext<B> for A where A: Context, B: Context {
    #[inline(always)]
    fn combine_with<'a>(self: &'a mut Self, other: &'a B) -> CombinedContextWrapper<'a, Self, B> {
        CombinedContextWrapper::new(self, other)
    }
}

impl<'a, A, B> Context for CombinedContextWrapper<'a, A, B> where A: Context, B: Context {
    fn get_value(&self, identifier: &str) -> Option<&Value> {
        match self.local_context.get_value(identifier) {
            None => {
                self.global_context.get_value(identifier)
            }
            Some(value) => {
                Some(value)
            }
        }
    }

    fn call_function(&self, identifier: &str, argument: &Value) -> EvalexprResult<Value> {
        match self.local_context.call_function(identifier, argument) {
            Ok(value) => {
                Ok(value)
            }
            Err(EvalexprError::FunctionIdentifierNotFound(_)) => {
                self.global_context.call_function(identifier, argument)
            }
            Err(EvalexprError::WrongFunctionArgumentAmount {..}) => {
                self.global_context.call_function(identifier, argument)
            }
            other => other
        }
    }

    fn are_builtin_functions_disabled(&self) -> bool {
        self.local_context.are_builtin_functions_disabled()
    }

    fn set_builtin_functions_disabled(&mut self, disabled: bool) -> EvalexprResult<()> {
        self.local_context.set_builtin_functions_disabled(disabled)
    }
}

impl<'a, A, B> ContextWithMutableVariables for CombinedContextWrapper<'a, A, B> where A: ContextWithMutableVariables, B: Context  {
    fn set_value(&mut self, identifier: String, value: Value) -> EvalexprResult<()> {
        self.local_context.set_value(identifier, value)
    }
}

impl<'a, A, B> ContextWithMutableFunctions for CombinedContextWrapper<'a, A, B> where A: ContextWithMutableFunctions, B: Context  {
    fn set_function(&mut self, identifier: String, function: Function) -> EvalexprResult<()> {
        self.local_context.set_function(identifier, function)
    }
}

impl<'a, A, B> IterateVariablesContext for CombinedContextWrapper<'a, A, B> where A: IterateVariablesContext, B: IterateVariablesContext {
    type VariableIterator<'b> where Self: 'b = Chain<
        <A as IterateVariablesContext>::VariableIterator<'b>,
        <B as IterateVariablesContext>::VariableIterator<'b>
    >;
    type VariableNameIterator<'b> where Self: 'b = Chain<
        <A as IterateVariablesContext>::VariableNameIterator<'b>,
        <B as IterateVariablesContext>::VariableNameIterator<'b>
    >;

    fn iter_variables(&self) -> Self::VariableIterator<'_> {
        self.local_context.iter_variables().chain(
            self.global_context.iter_variables()
        )
    }

    fn iter_variable_names(&self) -> Self::VariableNameIterator<'_> {
        self.local_context.iter_variable_names().chain(
            self.global_context.iter_variable_names()
        )
    }
}