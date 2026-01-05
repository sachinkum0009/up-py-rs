"""
Type stubs for up_py - Python bindings for up-rust (uProtocol).

This file provides type hints and IDE support for the up_py module.
"""

from typing import Optional, Callable

# Top-level classes

class UMessage:
    """
    Represents a complete uProtocol message.
    
    UMessage encapsulates both the payload and metadata for a uProtocol communication.
    It is typically received by listener callbacks.
    """
    
    def extract_string(self) -> Optional[str]:
        """
        Extract string value from the message payload.
        
        Returns:
            The extracted string if successful, None if the message
            doesn't contain a string value.
        
        Example:
            >>> text = message.extract_string()
            >>> if text:
            ...     print(f"Received: {text}")
        """
        ...


class StaticUriProvider:
    """
    Provides URI information for uProtocol entities.
    
    StaticUriProvider creates and manages URIs for identifying entities
    in the uProtocol network. It combines an authority (device/vehicle name),
    entity ID, and version to create unique identifiers.
    """
    
    def __init__(self, authority: str, entity_id: int, version: int) -> None:
        """
        Create a new StaticUriProvider.
        
        Args:
            authority: The authority name identifying the device/vehicle
                      (e.g., "my-vehicle", "sensor-001").
            entity_id: The entity ID (0 to 2^32-1, e.g., 0xa34b).
            version: The version number (0 to 255, e.g., 0x01).
        
        Example:
            >>> from up_py import StaticUriProvider
            >>> provider = StaticUriProvider("my-vehicle", 0xa34b, 0x01)
        """
        ...


# Submodule: communication

class communication:
    """Communication module containing publishers and payloads."""
    
    class UPayload:
        """
        Represents a message payload in uProtocol.
        
        UPayload encapsulates the data being transmitted in a uProtocol message.
        It can be created from strings or raw bytes.
        """
        
        @staticmethod
        def from_string(value: str) -> 'communication.UPayload':
            """
            Create a UPayload from a string value.
            
            Args:
                value: The string content to wrap in the payload.
            
            Returns:
                A new payload instance containing the string.
            
            Raises:
                Exception: If the payload creation fails.
            
            Example:
                >>> from up_py.communication import UPayload
                >>> payload = UPayload.from_string("Hello, World!")
            """
            ...
        
        @staticmethod
        def from_bytes(data: list[int]) -> 'communication.UPayload':
            """
            Create a UPayload from raw bytes.
            
            Args:
                data: A list of bytes (0-255) to wrap in the payload.
            
            Returns:
                A new payload instance containing the binary data.
            
            Example:
                >>> from up_py.communication import UPayload
                >>> payload = UPayload.from_bytes([72, 101, 108, 108, 111])
            """
            ...
    
    class SimplePublisher:
        """
        Publisher for sending uProtocol messages.
        
        SimplePublisher provides an easy-to-use interface for publishing messages
        to specific resources in the uProtocol network.
        """
        
        def __init__(self, transport: 'local_transport.LocalTransport', uri_provider: StaticUriProvider) -> None:
            """
            Create a new SimplePublisher.
            
            Args:
                transport: The transport to use for sending messages.
                uri_provider: The URI provider for the publishing entity.
            
            Raises:
                Exception: If the runtime creation fails.
            
            Example:
                >>> from up_py.communication import SimplePublisher
                >>> from up_py.local_transport import LocalTransport
                >>> from up_py import StaticUriProvider
                >>> transport = LocalTransport()
                >>> provider = StaticUriProvider("device", 0x1234, 0x01)
                >>> publisher = SimplePublisher(transport, provider)
            """
            ...
        
        def publish(self, resource_id: int, payload: Optional['communication.UPayload']) -> None:
            """
            Publish a message to a specific resource.
            
            Args:
                resource_id: The target resource ID (0 to 65535).
                payload: The message payload, or None for empty messages.
            
            Raises:
                Exception: If publishing fails.
            
            Example:
                >>> from up_py.communication import UPayload
                >>> payload = UPayload.from_string("Hello")
                >>> publisher.publish(0xb4c1, payload)
                >>> # Or publish without payload:
                >>> publisher.publish(0xb4c1, None)
            """
            ...


# Submodule: local_transport

class local_transport:
    """Local transport module for in-process communication."""
    
    class LocalTransport:
        """
        Provides local (in-process) transport for uProtocol communication.
        
        LocalTransport enables communication between components within the same
        process without network overhead. It manages listener registration and
        message routing.
        """
        
        def __init__(self) -> None:
            """
            Create a new LocalTransport instance.
            
            Raises:
                Exception: If the runtime creation fails.
            
            Example:
                >>> from up_py.local_transport import LocalTransport
                >>> transport = LocalTransport()
            """
            ...
        
        def register_listener(
            self,
            uri_provider: StaticUriProvider,
            resource_id: int,
            callback: Callable[[UMessage], None]
        ) -> None:
            """
            Register a listener callback for a specific resource.
            
            Args:
                uri_provider: The URI provider identifying the entity.
                resource_id: The resource ID to listen to (0 to 65535).
                callback: A Python function that accepts a UMessage parameter.
                         Will be called when messages arrive.
            
            Raises:
                Exception: If registration fails.
            
            Example:
                >>> from up_py import UMessage
                >>> def my_handler(msg: UMessage):
                ...     print(msg.extract_string())
                >>> transport.register_listener(uri_provider, 0xb4c1, my_handler)
            """
            ...
        
        def unregister_listener(
            self,
            uri_provider: StaticUriProvider,
            resource_id: int,
            callback: Callable[[UMessage], None]
        ) -> None:
            """
            Unregister a previously registered listener.
            
            Args:
                uri_provider: The URI provider used during registration.
                resource_id: The resource ID to stop listening to.
                callback: The same Python function that was registered.
            
            Raises:
                Exception: If unregistration fails or listener not found.
            
            Note:
                Currently may fail due to listener instance comparison issues.
                Consider letting listeners be cleaned up automatically.
            """
            ...


__all__ = [
    'UMessage',
    'StaticUriProvider',
    'communication',
    'local_transport',
]
