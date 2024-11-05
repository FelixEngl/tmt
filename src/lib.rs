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

extern crate core;

use pyo3::{Bound, pymodule, PyResult};
use pyo3::prelude::PyModule;

pub mod translate;
pub mod voting;
pub mod toolkit;
pub mod variable_names;
pub mod py;
mod external_variable_provider;
pub mod aligned_data;
pub mod tokenizer;
pub mod topicmodel;

/// A Python module implemented in Rust. The name of this function must match
/// the `lib.name` setting in the `Cargo.toml`, else Python will not be able to
/// import the module.
#[pymodule]
pub fn ldatranslate(m: &Bound<'_, PyModule>) -> PyResult<()> {
    for x in inventory::iter::<toolkit::register_python::PythonRegistration> {
        (&x.register)(m)?;
    }
    Ok(())
}

#[cfg(feature = "gen_python_api")]
pyo3_stub_gen::define_stub_info_gatherer!(stub_info);


#[cfg(test)]
mod test {
    use env_logger::{Target};
    use log::LevelFilter;
    use pyo3::prelude::*;

    #[test]
    fn can_register_the_modules(){
        let _ = env_logger::builder()
            .target(Target::Stdout)
            .filter_level(LevelFilter::Debug)
            .init();

        Python::with_gil(|py| {
            let ldatranslate = PyModule::new_bound(py, "ldatranslate")?;
            super::ldatranslate(&ldatranslate)
        }).unwrap();
    }
}