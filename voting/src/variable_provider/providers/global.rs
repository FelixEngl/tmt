use std::sync::{Arc, RwLock};
use evalexpr::{ContextWithMutableVariables, Value};
use crate::variable_provider::providers::{SharedInterner, VarName};
use crate::variable_provider::VariableProviderResult;
use crate::voting::constants::TMTNumericTypes;

#[derive(Debug, Clone)]
pub struct GlobalVariableProvider {
    provider: SharedInterner,
    variables: Arc<RwLock<Vec<(VarName, Value<TMTNumericTypes>)>>>
}

impl GlobalVariableProvider {
    pub(super) fn new(provider: SharedInterner) -> Self {
        Self { provider, variables: Default::default() }
    }

    pub fn register_variable(&self, key: impl AsRef<str>, value: impl Into<Value<TMTNumericTypes>>) -> VariableProviderResult<()> {
        self.variables.write().unwrap().push((self.provider.intern_fast(key), value.into()));
        Ok(())
    }

    pub fn provide_variables(&self, target: &mut impl ContextWithMutableVariables<NumericTypes=TMTNumericTypes>) -> VariableProviderResult<()> {
        let resolver = self.provider.resolver();
        for (k, v) in self.variables.read().unwrap().iter() {
            target.set_value(resolver.resolve(*k).to_string(), v.clone())?;
        }
        Ok(())
    }
}
