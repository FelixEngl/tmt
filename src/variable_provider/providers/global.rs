use std::sync::{Arc, RwLock};
use evalexpr::{Context, ContextWithMutableVariables, EvalexprNumericTypes, Value};
use crate::variable_provider::providers::{SharedInterner, VarName};
use crate::variable_provider::VariableProviderResult;

#[derive(Debug, Clone)]
pub struct GlobalVariableProvider<NumericTypes: EvalexprNumericTypes> {
    provider: SharedInterner,
    variables: Arc<RwLock<Vec<(VarName, Value<NumericTypes>)>>>
}

impl<NumericTypes: EvalexprNumericTypes> GlobalVariableProvider<NumericTypes> {
    pub(super) fn new(provider: SharedInterner) -> Self {
        Self { provider, variables: Default::default() }
    }

    pub fn register_variable(&self, key: impl AsRef<str>, value: impl Into<Value<NumericTypes>>) -> VariableProviderResult<(), NumericTypes> {
        self.variables.write().unwrap().push((self.provider.intern_fast(key), value.into()));
        Ok(())
    }

    pub fn provide_variables(&self, target: &mut (impl ContextWithMutableVariables + Context<NumericTypes=NumericTypes>)) -> VariableProviderResult<(), NumericTypes> {
        let resolver = self.provider.resolver();
        for (k, v) in self.variables.read().unwrap().iter() {
            target.set_value(resolver.resolve(*k).to_string(), v.clone())?;
        }
        Ok(())
    }
}
