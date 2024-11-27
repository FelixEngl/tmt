use std::marker::PhantomData;
use std::sync::{Arc, RwLock};
use evalexpr::{ContextWithMutableVariables, Value};
use crate::variable_provider::providers::{SharedInterner, VarName};
use crate::variable_provider::targets::Target;
use crate::variable_provider::VariableProviderResult;
use crate::voting::constants::TMTNumericTypes;

#[derive(Debug, Clone)]
pub struct IdBasedVariableProvider<T: Target> {
    provider: SharedInterner,
    variables: Arc<RwLock<Vec<Vec<(VarName, Value<TMTNumericTypes>)>>>>,
    _target_type: PhantomData<T>
}

impl<T> IdBasedVariableProvider<T> where T: Target {
    pub fn new(provider: SharedInterner, id_count: usize) -> Self {
        let mut per_topic = Vec::with_capacity(id_count);
        for _ in 0..per_topic.capacity() {
            per_topic.push(Vec::with_capacity(1))
        }
        Self {
            provider,
            variables: Arc::new(RwLock::new(per_topic)),
            _target_type: PhantomData
        }
    }

    pub fn register_variable(&self, id: usize, key: impl AsRef<str>, value: impl Into<Value<TMTNumericTypes>>) -> VariableProviderResult<()> {
        let mut variable_lock = self.variables.write().unwrap();
        if let Some(data) = variable_lock.get_mut(id) {
            data.push((self.provider.intern_fast(key), value.into()));
            Ok(())
        } else {
            Err(T::make_error(id, variable_lock.len()))
        }
    }

    pub fn provide_variables(&self, id: usize, target: &mut impl ContextWithMutableVariables<NumericTypes=TMTNumericTypes>) -> VariableProviderResult<()> {
        let variables = self.variables.read().unwrap();
        if let Some(for_id) = variables.get(id) {
            let resolver = self.provider.resolver();
            for (k, v) in for_id {
                target.set_value(resolver.resolve(*k).to_string(), v.clone())?;
            }
            Ok(())
        } else {
            Err(T::make_error(id, variables.len()))
        }
    }
}
