// ==============================================================================
// NetFreedom Cloudflare Worker: VLESS over WebSocket Relay
// Enables 100% encrypted, undetectable tunneling over standard HTTPS (TCP 443)
// Bypasses enterprise middleboxes (e.g. Fortinet) enforcing full TCP reassembly.
// ==============================================================================

import { connect } from 'cloudflare:sockets';

// Default User ID (UUIDv4). You can set this in Cloudflare Worker Environment Variables as 'UUID'
const DEFAULT_UUID = 'd342d110-d4a3-45c3-b0a3-3b11815b80b2';

export default {
    async fetch(request, env, ctx) {
        try {
            const userID = env.UUID || DEFAULT_UUID;
            const upgradeHeader = request.headers.get('Upgrade');

            if (!upgradeHeader || upgradeHeader.toLowerCase() !== 'websocket') {
                const url = new URL(request.url);
                switch (url.pathname) {
                    case '/':
                        return new Response(
                            JSON.stringify({
                                status: 'online',
                                service: 'NetFreedom VLESS Stealth Relay',
                                protocol: 'vless-ws-tls',
                                port: 443,
                                instructions: 'Connect via NetFreedom Desktop or Android using this worker domain.'
                            }, null, 2),
                            {
                                status: 200,
                                headers: { 'Content-Type': 'application/json' }
                            }
                        );
                    default:
                        return new Response('404 Not Found', { status: 404 });
                }
            }

            return await vlessOverWSHandler(request, userID);
        } catch (err) {
            return new Response(err.toString(), { status: 500 });
        }
    }
};

async function vlessOverWSHandler(request, userID) {
    const webSocketPair = new WebSocketPair();
    const [client, server] = Object.values(webSocketPair);
    server.accept();

    let remoteSocket = null;
    let isHeaderProcessed = false;

    // Support early data (?ed=2048 or Sec-WebSocket-Protocol)
    const url = new URL(request.url);
    const earlyDataHeader = request.headers.get('sec-websocket-protocol') || '';

    const readable = new ReadableStream({
        start(controller) {
            server.addEventListener('message', async (event) => {
                controller.enqueue(event.data);
            });
            server.addEventListener('close', () => {
                controller.close();
            });
            server.addEventListener('error', (err) => {
                controller.error(err);
            });

            if (earlyDataHeader) {
                try {
                    const earlyData = base64ToArrayBuffer(decodeURIComponent(earlyDataHeader));
                    if (earlyData && earlyData.byteLength > 0) {
                        controller.enqueue(earlyData);
                    }
                } catch (e) {
                    // Ignore malformed early data
                }
            }
        }
    });

    // Process stream in background
    (async () => {
        const reader = readable.getReader();
        let writer = null;

        try {
            while (true) {
                const { done, value } = await reader.read();
                if (done) break;

                const chunk = new Uint8Array(value);

                if (!isHeaderProcessed) {
                    // VLESS Header format:
                    // 1 byte: version (0)
                    // 16 bytes: UUID
                    // 1 byte: addon length (M)
                    // M bytes: addons
                    // 1 byte: command (1: TCP, 2: UDP)
                    // 2 bytes: port (big endian)
                    // 1 byte: address type (1: IPv4 [4B], 2: Domain [1B len + str], 3: IPv6 [16B])
                    if (chunk.length < 24) {
                        server.close(1002, 'VLESS header too short');
                        return;
                    }

                    const version = chunk[0];
                    if (version !== 0) {
                        server.close(1002, 'Unsupported VLESS version');
                        return;
                    }

                    const clientUUID = stringifyUUID(chunk.slice(1, 17));
                    if (clientUUID.toLowerCase() !== userID.toLowerCase()) {
                        server.close(1008, 'Unauthorized UUID');
                        return;
                    }

                    const addonLen = chunk[17];
                    let offset = 18 + addonLen;

                    const command = chunk[offset++];
                    if (command !== 1) { // 1 = TCP
                        server.close(1003, 'Only TCP connections supported in WebSocket relay');
                        return;
                    }

                    const port = (chunk[offset] << 8) | chunk[offset + 1];
                    offset += 2;

                    const addressType = chunk[offset++];
                    let address = '';

                    if (addressType === 1) {
                        // IPv4
                        address = `${chunk[offset]}.${chunk[offset + 1]}.${chunk[offset + 2]}.${chunk[offset + 3]}`;
                        offset += 4;
                    } else if (addressType === 2) {
                        // Domain name
                        const domainLen = chunk[offset++];
                        address = new TextDecoder().decode(chunk.slice(offset, offset + domainLen));
                        offset += domainLen;
                    } else if (addressType === 3) {
                        // IPv6
                        const v6Parts = [];
                        for (let i = 0; i < 8; i++) {
                            v6Parts.push(((chunk[offset + i * 2] << 8) | chunk[offset + i * 2 + 1]).toString(16));
                        }
                        address = v6Parts.join(':');
                        offset += 16;
                    } else {
                        server.close(1002, 'Invalid address type');
                        return;
                    }

                    // Send VLESS response header: [version: 0, addon length: 0]
                    server.send(new Uint8Array([0, 0]));

                    // Connect outbound TCP socket
                    remoteSocket = connect({
                        hostname: address,
                        port: port,
                    });

                    writer = remoteSocket.writable.getWriter();
                    isHeaderProcessed = true;

                    // Pipe remote socket back to WebSocket
                    remoteSocket.readable.pipeTo(new WritableStream({
                        write(data) {
                            if (server.readyState === WebSocket.OPEN) {
                                server.send(data);
                            }
                        },
                        close() {
                            if (server.readyState === WebSocket.OPEN) server.close();
                        },
                        abort() {
                            if (server.readyState === WebSocket.OPEN) server.close();
                        }
                    })).catch(() => {});

                    // If chunk had payload data after the VLESS header, forward it to remote
                    if (offset < chunk.length) {
                        await writer.write(chunk.slice(offset));
                    }
                } else if (writer) {
                    await writer.write(chunk);
                }
            }
        } catch (err) {
            // Socket closed or error
        } finally {
            if (writer) try { writer.close(); } catch (_) {}
            if (remoteSocket) try { remoteSocket.close(); } catch (_) {}
            if (server.readyState === WebSocket.OPEN) server.close();
        }
    })();

    return new Response(null, {
        status: 101,
        webSocket: client,
    });
}

function stringifyUUID(bytes) {
    const hex = Array.from(bytes).map(b => b.toString(16).padStart(2, '0')).join('');
    return `${hex.slice(0, 8)}-${hex.slice(8, 12)}-${hex.slice(12, 16)}-${hex.slice(16, 20)}-${hex.slice(20)}`;
}

function base64ToArrayBuffer(base64) {
    base64 = base64.replace(/-/g, '+').replace(/_/g, '/');
    while (base64.length % 4) base64 += '=';
    const binary = atob(base64);
    const bytes = new Uint8Array(binary.length);
    for (let i = 0; i < binary.length; i++) {
        bytes[i] = binary.charCodeAt(i);
    }
    return bytes.buffer;
}
