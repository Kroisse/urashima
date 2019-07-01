//  Copyright 2019 Eunchong Yu <kroisse@gmail.com>
//
//  Licensed under the Apache License, Version 2.0 (the "License");
//  you may not use this file except in compliance with the License.
//  You may obtain a copy of the License at
//
//      http://www.apache.org/licenses/LICENSE-2.0
//
//  Unless required by applicable law or agreed to in writing, software
//  distributed under the License is distributed on an "AS IS" BASIS,
//  WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
//  See the License for the specific language governing permissions and
//  limitations under the License.
#![warn(clippy::all)]
#![deny(clippy::correctness)]
#![deny(rust_2018_idioms)]

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc<'_> = wee_alloc::WeeAlloc::INIT;

use std::cell::RefCell;
use std::io::{self, prelude::*};

use urashima::Runtime;
use wasm_bindgen::prelude::*;
use web_sys::console;

#[wasm_bindgen]
#[derive(Default)]
pub struct NaruRuntime {
    rt: Runtime,
}

#[wasm_bindgen]
impl NaruRuntime {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        NaruRuntime { rt: Runtime::new() }
    }

    pub fn capsule(&self) -> Capsule {
        let capsule = self
            .rt
            .capsule_builder()
            .stdout(Box::new(ConsoleWriter))
            .build();
        Capsule {
            capsule: RefCell::new(capsule),
        }
    }
}

#[wasm_bindgen]
pub struct Capsule {
    capsule: RefCell<urashima::Capsule<'static>>,
}

#[wasm_bindgen]
impl Capsule {
    #[wasm_bindgen(catch, method)]
    pub fn eval(&self, code: &str) -> Result<(), JsValue> {
        let mut cap = self.capsule.borrow_mut();
        cap.eval(code).map_err(|e| e.to_string())?;
        Ok(())
    }
}

struct ConsoleWriter;

impl Write for ConsoleWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let s = String::from_utf8_lossy(buf);
        console::log_1(&JsValue::from(&*s));
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
