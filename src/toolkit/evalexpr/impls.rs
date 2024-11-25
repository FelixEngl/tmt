//Copyright 2024 Felix Engl
//
//Licensed under the Apache License, Version 2.0 (the "License");
//you may not use this file except in compliance with the License.
//You may obtain a copy of the License at
//
//    http://www.apache.org/licenses/LICENSE-2.0
//
//Unless required by applicable law or agreed to in writing, software
//distributed under the License is distributed on an "AS IS" BASIS,
//WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
//See the License for the specific language governing permissions and
//limitations under the License.

#![allow(dead_code)]

use std::iter::{Chain};
use std::sync::Arc;
use evalexpr::{Context, ContextWithMutableFunctions, ContextWithMutableVariables, EvalexprError, EvalexprResult, Function, HashMapContext, IterateVariablesContext, Value};
use evalexpr::EvalexprError::FunctionIdentifierNotFound;


#[derive(Debug)]
pub struct StaticContext<A: ?Sized, B: ?Sized> {
    current: Arc<A>,
    next: Arc<B>
}

unsafe impl<A, B> Sync for StaticContext<A, B>{}
unsafe impl<A, B> Send for StaticContext<A, B>{}

impl<A, B> Clone for StaticContext<A, B> {
    fn clone(&self) -> Self {
        Self {
            current: self.current.clone(),
            next: self.next.clone()
        }
    }
}


impl<A, B> StaticContext<A, B> where A: Context, B: Context {
    pub fn new(current: A, next: B) -> Self {
        Self { current: Arc::new(current), next: Arc::new(next) }
    }

    pub fn create_expanded<C: Context>(&self, other: C) -> StaticContext<C, StaticContext<A, B>> {
        StaticContext::<C, StaticContext<A, B>>::new(other, self.clone())
    }
}

impl<A, B> Context for StaticContext<A, B> where A: Context, B: Context {
    fn get_value(&self, identifier: &str) -> Option<&Value> {
        match self.current.get_value(identifier) {
            None => {
                self.next.get_value(identifier)
            }
            Some(value) => {
                Some(value)
            }
        }
    }

    fn call_function(&self, identifier: &str, argument: &Value) -> EvalexprResult<Value> {
        match self.current.call_function(identifier, argument) {
            Ok(value) => {
                Ok(value)
            }
            Err(FunctionIdentifierNotFound(_)) => {
                self.next.call_function(identifier, argument)
            }
            Err(EvalexprError::WrongFunctionArgumentAmount {..}) => {
                self.next.call_function(identifier, argument)
            }
            other => other
        }
    }

    fn are_builtin_functions_disabled(&self) -> bool {
        self.current.are_builtin_functions_disabled()
    }

    fn set_builtin_functions_disabled(&mut self, disabled: bool) -> EvalexprResult<()> {
        let are_disabled = self.are_builtin_functions_disabled();
        if disabled == are_disabled {
            Ok(())
        } else {
            if are_disabled {
                Err(EvalexprError::BuiltinFunctionsCannotBeEnabled)
            } else {
                Err(EvalexprError::BuiltinFunctionsCannotBeDisabled)
            }
        }
    }
}

impl<A, B> IterateVariablesContext for StaticContext<A, B> where A: IterateVariablesContext, B: IterateVariablesContext {
    type VariableIterator<'b> = Chain<
        <A as IterateVariablesContext>::VariableIterator<'b>,
        <B as IterateVariablesContext>::VariableIterator<'b>
    > where Self: 'b;
    type VariableNameIterator<'b> = Chain<
        <A as IterateVariablesContext>::VariableNameIterator<'b>,
        <B as IterateVariablesContext>::VariableNameIterator<'b>
    > where Self: 'b;
    fn iter_variables(&self) -> Self::VariableIterator<'_> {
        self.current.iter_variables().chain(
            self.next.iter_variables()
        )
    }

    fn iter_variable_names(&self) -> Self::VariableNameIterator<'_> {
        self.current.iter_variable_names().chain(
            self.next.iter_variable_names()
        )
    }
}

pub trait SimpleCombineableContext {
    fn as_empty_mutable<'a>(self: &'a Self) -> OwningContext<'a, HashMapContext, Self>;
}

impl<A> SimpleCombineableContext for A where A: Context {
    fn as_empty_mutable<'a>(self: &'a Self) -> OwningContext<'a, HashMapContext, Self> {
        HashMapContext::new().to_owning_with(self)
    }
}


pub trait CombineableContext<B> where B: Context {
    fn combine_with<'a>(self: &'a Self, other: &'a B) -> CombinedContextWrapper<'a, Self, B>;
    fn combine_with_mut<'a>(self: &'a mut Self, other: &'a B) -> CombinedContextWrapperMut<'a, Self, B>;
    fn to_static_with(self, other: B) -> StaticContext<Self, B>;
    fn to_owning_with<'a>(self, other: &'a B) -> OwningContext<'a, Self, B> where Self: Sized;
}


impl<A, B> CombineableContext<B> for A where A: Context, B: Context {
    fn combine_with<'a>(self: &'a Self, other: &'a B) -> CombinedContextWrapper<'a, Self, B> {
        CombinedContextWrapper::new(self, other)
    }

    #[inline(always)]
    fn combine_with_mut<'a>(self: &'a mut Self, other: &'a B) -> CombinedContextWrapperMut<'a, Self, B> {
        CombinedContextWrapperMut::new(self, other)
    }

    fn to_static_with(self, other: B) -> StaticContext<Self, B> {
        StaticContext::new(self, other)
    }

    fn to_owning_with<'a>(self, other: &'a B) -> OwningContext<'a, Self, B> where Self: Sized {
        OwningContext::new(self, other)
    }


}


/// Combines a global and a local context in a meaningful way.
/// Owns the local context
#[derive(Debug)]
pub struct OwningContext<'a, A: Sized, B: ?Sized> {
    local_context: A,
    global_context: &'a B
}

impl<'a, A, B> OwningContext<'a, A, B> {
    #[inline(always)]
    pub fn new(local_context: A, global_context: &'a B) -> Self {
        Self { local_context, global_context }
    }
}

impl<'a, A, B> Context for OwningContext<'a, A, B> where A: Context, B: Context {
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

impl<'a, A, B> ContextWithMutableVariables for OwningContext<'a, A, B> where A: ContextWithMutableVariables, B: Context  {
    fn set_value(&mut self, identifier: String, value: Value) -> EvalexprResult<()> {
        self.local_context.set_value(identifier, value)
    }
}

impl<'a, A, B> ContextWithMutableFunctions for OwningContext<'a, A, B> where A: ContextWithMutableFunctions, B: Context  {
    fn set_function(&mut self, identifier: String, function: Function) -> EvalexprResult<()> {
        self.local_context.set_function(identifier, function)
    }
}

impl<'a, A, B> IterateVariablesContext for OwningContext<'a, A, B> where A: IterateVariablesContext, B: IterateVariablesContext {
    type VariableIterator<'b> = Chain<
        <A as IterateVariablesContext>::VariableIterator<'b>,
        <B as IterateVariablesContext>::VariableIterator<'b>
    > where Self: 'b;
    type VariableNameIterator<'b> = Chain<
        <A as IterateVariablesContext>::VariableNameIterator<'b>,
        <B as IterateVariablesContext>::VariableNameIterator<'b>
    > where Self: 'b;

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






/// Combines a global and a local context in a meaningful way.
/// Borrows the local context in a mutable way
#[derive(Debug)]
pub struct CombinedContextWrapperMut<'a, A: ?Sized, B: ?Sized> {
    local_context: &'a mut A,
    global_context: &'a B
}

impl<'a, A, B> CombinedContextWrapperMut<'a, A, B> {
    #[inline(always)]
    pub fn new(local_context: &'a mut A, global_context: &'a B) -> Self {
        Self { local_context, global_context }
    }
}

impl<'a, A, B> Context for CombinedContextWrapperMut<'a, A, B> where A: Context, B: Context {
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

impl<'a, A, B> ContextWithMutableVariables for CombinedContextWrapperMut<'a, A, B> where A: ContextWithMutableVariables, B: Context  {
    fn set_value(&mut self, identifier: String, value: Value) -> EvalexprResult<()> {
        self.local_context.set_value(identifier, value)
    }
}

impl<'a, A, B> ContextWithMutableFunctions for CombinedContextWrapperMut<'a, A, B> where A: ContextWithMutableFunctions, B: Context  {
    fn set_function(&mut self, identifier: String, function: Function) -> EvalexprResult<()> {
        self.local_context.set_function(identifier, function)
    }
}

impl<'a, A, B> IterateVariablesContext for CombinedContextWrapperMut<'a, A, B> where A: IterateVariablesContext, B: IterateVariablesContext {
    type VariableIterator<'b> = Chain<
        <A as IterateVariablesContext>::VariableIterator<'b>,
        <B as IterateVariablesContext>::VariableIterator<'b>
    > where Self: 'b;
    type VariableNameIterator<'b> = Chain<
        <A as IterateVariablesContext>::VariableNameIterator<'b>,
        <B as IterateVariablesContext>::VariableNameIterator<'b>
    > where Self: 'b;

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



/// Conbines a global and a local context in a meaningful way.
#[derive(Debug, Clone)]
pub struct CombinedContextWrapper<'a, A: ?Sized, B: ?Sized> {
    local_context: &'a A,
    global_context: &'a B
}

impl<'a, A, B> CombinedContextWrapper<'a, A, B> {
    #[inline(always)]
    pub fn new(local_context: &'a A, global_context: &'a B) -> Self {
        Self { local_context, global_context }
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
        let builtin_functions_disabled = self.local_context.are_builtin_functions_disabled();
        if disabled == builtin_functions_disabled {
            Ok(())
        } else {
            if builtin_functions_disabled {
                Err(EvalexprError::BuiltinFunctionsCannotBeDisabled)
            } else {
                Err(EvalexprError::BuiltinFunctionsCannotBeEnabled)
            }
        }
    }
}

impl<'a, A, B> IterateVariablesContext for CombinedContextWrapper<'a, A, B> where A: IterateVariablesContext, B: IterateVariablesContext {
    type VariableIterator<'b> = Chain<
        <A as IterateVariablesContext>::VariableIterator<'b>,
        <B as IterateVariablesContext>::VariableIterator<'b>
    > where Self: 'b;
    type VariableNameIterator<'b> = Chain<
        <A as IterateVariablesContext>::VariableNameIterator<'b>,
        <B as IterateVariablesContext>::VariableNameIterator<'b>
    > where Self: 'b;

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


