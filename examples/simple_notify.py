"""
Simple Notify
"""

from up_py_rs.communication import SimpleNotifier
from up_py_rs.local_transport import StaticUriProvider, LocalTransport

def main():
    ORIGIN_RESOURCE_ID = 0xd100
    uri_provider = StaticUriProvider("my-vehicle", 0xa34b, 0x01)
    transport = LocalTransport()
    notifier = SimpleNotifier(transport, uri_provider)

    topic = uri_provider.get_resource_uri(ORIGIN_RESOURCE_ID)

    notifier.start_listening(str(topic))

    notifier.stop_listening(str(topic))

if __name__ == '__main__':
    main()
