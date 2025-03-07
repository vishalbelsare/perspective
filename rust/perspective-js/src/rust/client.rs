// ┏━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┓
// ┃ ██████ ██████ ██████       █      █      █      █      █ █▄  ▀███ █       ┃
// ┃ ▄▄▄▄▄█ █▄▄▄▄▄ ▄▄▄▄▄█  ▀▀▀▀▀█▀▀▀▀▀ █ ▀▀▀▀▀█ ████████▌▐███ ███▄  ▀█ █ ▀▀▀▀▀ ┃
// ┃ █▀▀▀▀▀ █▀▀▀▀▀ █▀██▀▀ ▄▄▄▄▄ █ ▄▄▄▄▄█ ▄▄▄▄▄█ ████████▌▐███ █████▄   █ ▄▄▄▄▄ ┃
// ┃ █      ██████ █  ▀█▄       █ ██████      █      ███▌▐███ ███████▄ █       ┃
// ┣━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┫
// ┃ Copyright (c) 2017, the Perspective Authors.                              ┃
// ┃ ╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌ ┃
// ┃ This file is part of the Perspective library, distributed under the terms ┃
// ┃ of the [Apache License 2.0](https://www.apache.org/licenses/LICENSE-2.0). ┃
// ┗━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┛

use std::error::Error;
use std::future::Future;
use std::sync::Arc;

use derivative::Derivative;
use futures::channel::oneshot;
use futures::future::LocalBoxFuture;
use js_sys::{Function, Uint8Array};
use macro_rules_attribute::apply;
#[cfg(doc)]
use perspective_client::SystemInfo;
use perspective_client::{ReconnectCallback, Session, TableData, TableInitOptions};
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::{future_to_promise, JsFuture};

pub use crate::table::*;
use crate::utils::{inherit_docs, ApiError, ApiResult, JsValueSerdeExt, LocalPollLoop};

#[wasm_bindgen]
extern "C" {
    #[derive(Clone)]
    #[wasm_bindgen(typescript_type = "TableInitOptions")]
    pub type JsTableInitOptions;
}

#[wasm_bindgen]
#[derive(Clone)]
pub struct ProxySession(perspective_client::ProxySession);

#[wasm_bindgen]
impl ProxySession {
    #[wasm_bindgen(constructor)]
    pub fn new(client: &Client, on_response: &Function) -> Self {
        let poll_loop = LocalPollLoop::new({
            let on_response = on_response.clone();
            move |msg: Vec<u8>| {
                let msg = Uint8Array::from(&msg[..]);
                on_response.call1(&JsValue::UNDEFINED, &JsValue::from(msg))?;
                Ok(JsValue::null())
            }
        });
        // NB: This swallows any errors raised by the inner callback
        let on_response = Box::new(move |msg: &[u8]| {
            wasm_bindgen_futures::spawn_local(poll_loop.poll(msg.to_vec()));
            Ok(())
        });
        Self(perspective_client::ProxySession::new(
            client.client.clone(),
            on_response,
        ))
    }

    #[wasm_bindgen]
    pub async fn handle_request(&self, value: JsValue) -> ApiResult<()> {
        let uint8array = Uint8Array::new(&value);
        let slice = uint8array.to_vec();
        self.0.handle_request(&slice).await?;
        Ok(())
    }

    #[wasm_bindgen]
    pub async fn poll(&self) -> ApiResult<()> {
        self.0.poll().await?;
        Ok(())
    }

    pub async fn close(self) {
        self.0.close().await;
    }
}

#[apply(inherit_docs)]
#[inherit_doc = "client.md"]
#[wasm_bindgen]
pub struct Client {
    pub(crate) close: Option<Function>,
    pub(crate) client: perspective_client::Client,
}

/// A wrapper around [`js_sys::Function`] to ease async integration for the
/// `reconnect` argument of [`Client::on_error`] callback.
#[derive(Derivative)]
#[derivative(Clone(bound = ""))]
struct JsReconnect<I>(Arc<dyn Fn(I) -> js_sys::Promise>);

unsafe impl<I> Send for JsReconnect<I> {}
unsafe impl<I> Sync for JsReconnect<I> {}

impl<I> JsReconnect<I> {
    fn run(&self, args: I) -> js_sys::Promise {
        self.0(args)
    }

    fn run_all(
        &self,
        args: I,
    ) -> impl Future<Output = Result<(), Box<dyn Error + Send + Sync + 'static>>> + Send + Sync + 'static
    {
        let (sender, receiver) = oneshot::channel::<Result<(), Box<dyn Error + Send + Sync>>>();
        let p = self.0(args);
        let _ = future_to_promise(async move {
            let result = JsFuture::from(p)
                .await
                .map(|_| ())
                .map_err(|x| format!("{:?}", x).into());

            sender.send(result).unwrap();
            Ok(JsValue::UNDEFINED)
        });

        async move { receiver.await.unwrap() }
    }
}

impl<F, I> From<F> for JsReconnect<I>
where
    F: Fn(I) -> js_sys::Promise + 'static,
{
    fn from(value: F) -> Self {
        JsReconnect(Arc::new(value))
    }
}

#[wasm_bindgen]
impl Client {
    #[wasm_bindgen(constructor)]
    pub fn new(send_request: Function, close: Option<Function>) -> Self {
        let send_request = JsReconnect::from(move |mut v: Vec<u8>| {
            let buff2 = unsafe { js_sys::Uint8Array::view_mut_raw(v.as_mut_ptr(), v.len()) };
            send_request
                .call1(&JsValue::UNDEFINED, &buff2)
                .unwrap()
                .unchecked_into::<js_sys::Promise>()
        });

        let client = perspective_client::Client::new_with_callback(move |msg| {
            let vec = msg.to_vec();
            Box::pin(send_request.run_all(vec))
        });

        Client { close, client }
    }

    #[wasm_bindgen]
    pub fn new_proxy_session(&self, on_response: &Function) -> ProxySession {
        ProxySession::new(self, on_response)
    }

    #[wasm_bindgen]
    pub async fn init(&self) -> ApiResult<()> {
        self.client.clone().init().await?;
        Ok(())
    }

    #[doc(hidden)]
    #[wasm_bindgen]
    pub async fn handle_response(&self, value: &JsValue) -> ApiResult<()> {
        let uint8array = Uint8Array::new(value);
        let slice = uint8array.to_vec();
        self.client.handle_response(&slice).await?;
        Ok(())
    }

    #[doc(hidden)]
    #[wasm_bindgen]
    pub async fn handle_error(
        &self,
        error: Option<String>,
        reconnect: Option<Function>,
    ) -> ApiResult<()> {
        self.client
            .handle_error(
                error,
                reconnect.map(|reconnect| {
                    let reconnect =
                        JsReconnect::from(move |()| match reconnect.call0(&JsValue::UNDEFINED) {
                            Ok(x) => x.unchecked_into::<js_sys::Promise>(),
                            Err(e) => {
                                // This error may occur when _invoking_ the function
                                tracing::warn!("{:?}", e);
                                js_sys::Promise::reject(&format!("C {:?}", e).into())
                            },
                        });

                    Arc::new(move || {
                        let fut = JsFuture::from(reconnect.run(()));
                        Box::pin(async move {
                            // This error may occur when _awaiting_ the promise returned by the
                            // function
                            if let Err(e) = fut.await {
                                if let Some(e) = e.dyn_ref::<js_sys::Object>() {
                                    Err(e.to_string().as_string().unwrap().into())
                                } else {
                                    Err(e.as_string().unwrap().into())
                                }
                            } else {
                                Ok(())
                            }
                        })
                            as LocalBoxFuture<'static, Result<(), Box<dyn Error>>>
                    }) as Arc<(dyn Fn() -> _ + Send + Sync)>
                }),
            )
            .await?;

        Ok(())
    }

    #[doc(hidden)]
    #[wasm_bindgen]
    pub async fn on_error(&self, callback: Function) -> ApiResult<u32> {
        let callback = JsReconnect::from(
            move |(message, reconnect): (Option<String>, Option<ReconnectCallback>)| {
                let cl: Closure<dyn Fn() -> js_sys::Promise> = Closure::new(move || {
                    let reconnect = reconnect.clone();
                    future_to_promise(async move {
                        if let Some(f) = reconnect {
                            f().await.map_err(|e| JsValue::from(format!("A {}", e)))?;
                        }

                        Ok(JsValue::UNDEFINED)
                    })
                });

                if let Err(e) = callback.call2(
                    &JsValue::UNDEFINED,
                    &JsValue::from(message),
                    &cl.into_js_value(),
                ) {
                    tracing::warn!("D {:?}", e);
                }

                js_sys::Promise::resolve(&JsValue::UNDEFINED)
            },
        );

        let id = self
            .client
            .on_error(Box::new(move |message, reconnect| {
                let callback = callback.clone();
                Box::pin(async move {
                    let _promise = callback.run((message, reconnect));
                    Ok(())
                })
            }))
            .await?;

        Ok(id)
    }

    #[apply(inherit_docs)]
    #[inherit_doc = "client/table.md"]
    #[wasm_bindgen]
    pub async fn table(
        &self,
        value: &JsTableInitData,
        options: Option<JsTableInitOptions>,
    ) -> ApiResult<Table> {
        let options = options
            .into_serde_ext::<Option<TableInitOptions>>()?
            .unwrap_or_default();

        let args = TableData::from_js_value(value, options.format)?;
        Ok(Table(self.client.table(args, options).await?))
    }

    #[apply(inherit_docs)]
    #[inherit_doc = "client/terminate.md"]
    #[wasm_bindgen]
    pub fn terminate(&self) -> ApiResult<JsValue> {
        if let Some(f) = self.close.clone() {
            Ok(f.call0(&JsValue::UNDEFINED)?)
        } else {
            Err(ApiError::new("Client type cannot be terminated"))
        }
    }

    #[apply(inherit_docs)]
    #[inherit_doc = "client/open_table.md"]
    #[wasm_bindgen]
    pub async fn open_table(&self, entity_id: String) -> ApiResult<Table> {
        Ok(Table(self.client.open_table(entity_id).await?))
    }

    #[apply(inherit_docs)]
    #[inherit_doc = "client/get_hosted_table_names.md"]
    #[wasm_bindgen]
    pub async fn get_hosted_table_names(&self) -> ApiResult<JsValue> {
        Ok(JsValue::from_serde_ext(
            &self.client.get_hosted_table_names().await?,
        )?)
    }

    #[apply(inherit_docs)]
    #[inherit_doc = "client/system_info.md"]
    #[wasm_bindgen]
    pub async fn system_info(&self) -> ApiResult<JsValue> {
        let info = self.client.system_info().await?;
        Ok(JsValue::from_serde_ext(&info)?)
    }
}
