"""
Simple Notify Example

This example demonstrates how to use SimpleNotifier to send and receive
notifications in uProtocol. It creates a notifier, registers a listener,
sends a notification to itself, and then cleans up.
"""

from up_py_rs.communication import SimpleNotifier, UPayload
from up_py_rs import StaticUriProvider
from up_py_rs.local_transport import LocalTransport

def console_printer(msg):
    """
    Callback function that prints received notification messages.
    
    Args:
        msg: The UMessage containing the notification payload.
    """
    text = msg.extract_string()
    if text:
        print(f"Received notification: {text}")

def main():
    # Resource ID for the notification origin
    ORIGIN_RESOURCE_ID = 0xd100
    
    # Create URI provider for "my-vehicle" entity
    uri_provider = StaticUriProvider("my-vehicle", 0xa34b, 0x01)
    
    # Create local transport (for same-process communication)
    transport = LocalTransport()
    
    # Create notifier instance
    notifier = SimpleNotifier(transport, uri_provider)
    
    # Get the topic URI for notifications
    topic = uri_provider.get_resource_uri(ORIGIN_RESOURCE_ID)
    
    # Start listening for notifications on the topic
    print(f"Starting to listen for notifications...")
    notifier.start_listening(topic, console_printer)
    
    # Create a notification payload
    payload = UPayload.from_string("Hello from Python!")
    
    # Send notification to ourselves (using source URI as destination)
    destination = uri_provider.get_source_uri()
    print(f"Sending notification...")
    notifier.notify(ORIGIN_RESOURCE_ID, destination, payload)
    
    # Stop listening (cleanup)
    # Note: Unregistration may fail due to listener instance comparison issues
    # This is a known limitation documented in the copilot instructions
    print(f"Stopping listener...")
    try:
        notifier.stop_listening(topic, console_printer)
    except Exception as e:
        print(f"Note: Listener cleanup returned: {e}")
        print("This is a known limitation - listener will be cleaned up automatically")
    
    print("Done!")

if __name__ == '__main__':
    main()
