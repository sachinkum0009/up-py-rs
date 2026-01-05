use pyo3::prelude::*;
use pyo3::exceptions::PyException;
use std::sync::Arc;
use up_rust::{
    communication::{CallOptions, Publisher, SimplePublisher as RustSimplePublisher, UPayload as RustUPayload},
    local_transport::LocalTransport as RustLocalTransport,
    LocalUriProvider, StaticUriProvider as RustStaticUriProvider, UListener, UMessage as RustUMessage, UTransport,
    UPayloadFormat,
};
use protobuf::well_known_types::wrappers::StringValue;

/// Represents a message payload in uProtocol.
/// 
/// UPayload encapsulates the data being transmitted in a uProtocol message.
/// It can be created from strings or raw bytes.
#[pyclass]
#[derive(Clone)]
pub struct UPayload {
    inner: RustUPayload,
}

#[pymethods]
impl UPayload {
    /// Create a UPayload from a string value.
    /// 
    /// Args:
    ///     value (str): The string content to wrap in the payload.
    /// 
    /// Returns:
    ///     UPayload: A new payload instance containing the string.
    /// 
    /// Raises:
    ///     Exception: If the payload creation fails.
    /// 
    /// Example:
    ///     >>> payload = up_py.UPayload.from_string("Hello, World!")
    #[staticmethod]
    fn from_string(value: String) -> PyResult<Self> {
        let string_value = StringValue {
            value,
            ..Default::default()
        };
        let payload = RustUPayload::try_from_protobuf(string_value)
            .map_err(|e| PyException::new_err(format!("Failed to create payload: {}", e)))?;
        Ok(UPayload { inner: payload })
    }

    /// Create a UPayload from raw bytes.
    /// 
    /// Args:
    ///     data (list[int]): A list of bytes (0-255) to wrap in the payload.
    /// 
    /// Returns:
    ///     UPayload: A new payload instance containing the binary data.
    /// 
    /// Example:
    ///     >>> payload = up_py.UPayload.from_bytes([72, 101, 108, 108, 111])
    #[staticmethod]
    fn from_bytes(data: Vec<u8>) -> PyResult<Self> {
        // Create a simple bytes payload with a generic format
        Ok(UPayload {
            inner: RustUPayload::new(data, UPayloadFormat::UPAYLOAD_FORMAT_PROTOBUF_WRAPPED_IN_ANY),
        })
    }
}

/// Represents a complete uProtocol message.
/// 
/// UMessage encapsulates both the payload and metadata for a uProtocol communication.
/// It is typically received by listener callbacks.
#[pyclass]
#[derive(Clone)]
pub struct UMessage {
    inner: RustUMessage,
}

#[pymethods]
impl UMessage {
    /// Extract string value from the message payload.
    /// 
    /// Returns:
    ///     str | None: The extracted string if successful, None if the message
    ///                 doesn't contain a string value.
    /// 
    /// Example:
    ///     >>> text = message.extract_string()
    ///     >>> if text:
    ///     ...     print(f"Received: {text}")
    fn extract_string(&self) -> PyResult<Option<String>> {
        match self.inner.extract_protobuf::<StringValue>() {
            Ok(value) => Ok(Some(value.value)),
            Err(_) => Ok(None),
        }
    }
}

/// Provides URI information for uProtocol entities.
/// 
/// StaticUriProvider creates and manages URIs for identifying entities
/// in the uProtocol network. It combines an authority (device/vehicle name),
/// entity ID, and version to create unique identifiers.
#[pyclass]
pub struct StaticUriProvider {
    inner: Arc<RustStaticUriProvider>,
}

#[pymethods]
impl StaticUriProvider {
    /// Create a new StaticUriProvider.
    /// 
    /// Args:
    ///     authority (str): The authority name identifying the device/vehicle
    ///                     (e.g., "my-vehicle", "sensor-001").
    ///     entity_id (int): The entity ID (0 to 2^32-1, e.g., 0xa34b).
    ///     version (int): The version number (0 to 255, e.g., 0x01).
    /// 
    /// Returns:
    ///     StaticUriProvider: A new URI provider instance.
    /// 
    /// Example:
    ///     >>> provider = up_py.StaticUriProvider("my-vehicle", 0xa34b, 0x01)
    #[new]
    fn new(authority: String, entity_id: u32, version: u8) -> Self {
        StaticUriProvider {
            inner: Arc::new(RustStaticUriProvider::new(&authority, entity_id, version)),
        }
    }
}

/// Provides local (in-process) transport for uProtocol communication.
/// 
/// LocalTransport enables communication between components within the same
/// process without network overhead. It manages listener registration and
/// message routing.
#[pyclass]
pub struct LocalTransport {
    inner: Arc<RustLocalTransport>,
    runtime: tokio::runtime::Runtime,
}

#[pymethods]
impl LocalTransport {
    /// Create a new LocalTransport instance.
    /// 
    /// Returns:
    ///     LocalTransport: A new transport instance.
    /// 
    /// Raises:
    ///     Exception: If the runtime creation fails.
    /// 
    /// Example:
    ///     >>> transport = up_py.LocalTransport()
    #[new]
    fn new() -> PyResult<Self> {
        let runtime = tokio::runtime::Runtime::new()
            .map_err(|e| PyException::new_err(format!("Failed to create runtime: {}", e)))?;
        Ok(LocalTransport {
            inner: Arc::new(RustLocalTransport::default()),
            runtime,
        })
    }

    /// Register a listener callback for a specific resource.
    /// 
    /// Args:
    ///     uri_provider (StaticUriProvider): The URI provider identifying the entity.
    ///     resource_id (int): The resource ID to listen to (0 to 65535).
    ///     callback (callable): A Python function that accepts a UMessage parameter.
    ///                         Will be called when messages arrive.
    /// 
    /// Raises:
    ///     Exception: If registration fails.
    /// 
    /// Example:
    ///     >>> def my_handler(msg: UMessage):
    ///     ...     print(msg.extract_string())
    ///     >>> transport.register_listener(uri_provider, 0xb4c1, my_handler)
    fn register_listener(
        &mut self,
        _py: Python,
        uri_provider: &StaticUriProvider,
        resource_id: u16,
        callback: PyObject,
    ) -> PyResult<()> {
        let listener = Arc::new(PythonListener {
            callback: callback.clone(),
        });
        let uri = uri_provider.inner.get_resource_uri(resource_id);
        
        self.runtime.block_on(async {
            self.inner
                .register_listener(&uri, None, listener)
                .await
                .map_err(|e| PyException::new_err(format!("Failed to register listener: {}", e)))
        })
    }

    /// Unregister a previously registered listener.
    /// 
    /// Args:
    ///     uri_provider (StaticUriProvider): The URI provider used during registration.
    ///     resource_id (int): The resource ID to stop listening to.
    ///     callback (callable): The same Python function that was registered.
    /// 
    /// Raises:
    ///     Exception: If unregistration fails or listener not found.
    /// 
    /// Note:
    ///     Currently may fail due to listener instance comparison issues.
    ///     Consider letting listeners be cleaned up automatically.
    fn unregister_listener(
        &mut self,
        _py: Python,
        uri_provider: &StaticUriProvider,
        resource_id: u16,
        callback: PyObject,
    ) -> PyResult<()> {
        let listener = Arc::new(PythonListener {
            callback: callback.clone(),
        });
        let uri = uri_provider.inner.get_resource_uri(resource_id);
        
        self.runtime.block_on(async {
            self.inner
                .unregister_listener(&uri, None, listener)
                .await
                .map_err(|e| PyException::new_err(format!("Failed to unregister listener: {}", e)))
        })
    }
}

/// Internal struct to bridge Python callbacks to Rust UListener trait
struct PythonListener {
    callback: PyObject,
}

#[async_trait::async_trait]
impl UListener for PythonListener {
    async fn on_receive(&self, msg: RustUMessage) {
        Python::with_gil(|py| {
            let py_msg = UMessage { inner: msg };
            if let Err(e) = self.callback.call1(py, (py_msg,)) {
                eprintln!("Error calling Python callback: {:?}", e);
            }
        });
    }
}

/// Publisher for sending uProtocol messages.
/// 
/// SimplePublisher provides an easy-to-use interface for publishing messages
/// to specific resources in the uProtocol network.
#[pyclass]
pub struct SimplePublisher {
    inner: RustSimplePublisher,
    runtime: tokio::runtime::Runtime,
}

#[pymethods]
impl SimplePublisher {
    /// Create a new SimplePublisher.
    /// 
    /// Args:
    ///     transport (LocalTransport): The transport to use for sending messages.
    ///     uri_provider (StaticUriProvider): The URI provider for the publishing entity.
    /// 
    /// Returns:
    ///     SimplePublisher: A new publisher instance.
    /// 
    /// Raises:
    ///     Exception: If the runtime creation fails.
    /// 
    /// Example:
    ///     >>> transport = up_py.LocalTransport()
    ///     >>> provider = up_py.StaticUriProvider("device", 0x1234, 0x01)
    ///     >>> publisher = up_py.SimplePublisher(transport, provider)
    #[new]
    fn new(transport: &LocalTransport, uri_provider: &StaticUriProvider) -> PyResult<Self> {
        let runtime = tokio::runtime::Runtime::new()
            .map_err(|e| PyException::new_err(format!("Failed to create runtime: {}", e)))?;
        Ok(SimplePublisher {
            inner: RustSimplePublisher::new(transport.inner.clone(), uri_provider.inner.clone()),
            runtime,
        })
    }

    /// Publish a message to a specific resource.
    /// 
    /// Args:
    ///     resource_id (int): The target resource ID (0 to 65535).
    ///     payload (UPayload | None): The message payload, or None for empty messages.
    /// 
    /// Raises:
    ///     Exception: If publishing fails.
    /// 
    /// Example:
    ///     >>> payload = up_py.UPayload.from_string("Hello")
    ///     >>> publisher.publish(0xb4c1, payload)
    ///     >>> # Or publish without payload:
    ///     >>> publisher.publish(0xb4c1, None)
    fn publish(&mut self, _py: Python, resource_id: u16, payload: Option<UPayload>) -> PyResult<()> {
        let payload_inner = payload.map(|p| p.inner);
        let call_options = CallOptions::for_publish(None, None, None);
        
        self.runtime.block_on(async {
            self.inner
                .publish(resource_id, call_options, payload_inner)
                .await
                .map_err(|e| PyException::new_err(format!("Failed to publish: {}", e)))
        })
    }
}

#[pymodule]
fn up_py(py: Python, m: &PyModule) -> PyResult<()> {
    // Create and register submodules
    let communication_mod = PyModule::new(py, "communication")?;
    communication_mod.add_class::<UPayload>()?;
    communication_mod.add_class::<SimplePublisher>()?;
    m.add_submodule(communication_mod)?;
    py.import("sys")?
        .getattr("modules")?
        .set_item("up_py.communication", communication_mod)?;
    
    let local_transport_mod = PyModule::new(py, "local_transport")?;
    local_transport_mod.add_class::<LocalTransport>()?;
    m.add_submodule(local_transport_mod)?;
    py.import("sys")?
        .getattr("modules")?
        .set_item("up_py.local_transport", local_transport_mod)?;
    
    // Add top-level classes
    m.add_class::<UMessage>()?;
    m.add_class::<StaticUriProvider>()?;
    
    Ok(())
}
