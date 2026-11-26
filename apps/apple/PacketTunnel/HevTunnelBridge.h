#ifndef HevTunnelBridge_h
#define HevTunnelBridge_h

#include <stddef.h>

#ifdef __cplusplus
extern "C" {
#endif

/// Starts the HevSocks5Tunnel reading packets from the provided TUN file descriptor
/// and translating them to the local SOCKS5 proxy at 127.0.0.1:1080.
int hev_socks5_tunnel_main(const char *config_path, int tun_fd);

/// Stops the active tunnel loop.
void hev_socks5_tunnel_quit(void);

#ifdef __cplusplus
}
#endif

#endif /* HevTunnelBridge_h */
