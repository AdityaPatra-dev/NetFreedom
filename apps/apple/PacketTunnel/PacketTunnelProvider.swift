import NetworkExtension
import Foundation

class PacketTunnelProvider: NEPacketTunnelProvider {
    private var isRunning = false

    override func startTunnel(options: [String: NSObject]?, completionHandler: @escaping (Error?) -> Void) {
        let settings = NEPacketTunnelNetworkSettings(tunnelRemoteAddress: "127.0.0.1")

        // Configure IPv4 routing: Route all device traffic through virtual tunnel
        let ipv4Settings = NEIPv4Settings(addresses: ["10.0.0.2"], subnetMasks: ["255.255.255.0"])
        ipv4Settings.includedRoutes = [NEIPv4Route.default()]
        settings.ipv4Settings = ipv4Settings

        // Configure DNS settings (force 1.1.1.1 to prevent campus DNS leaks)
        let dnsSettings = NEDNSSettings(servers: ["1.1.1.1", "1.0.0.1"])
        dnsSettings.matchDomains = [""]
        settings.dnsSettings = dnsSettings

        settings.mtu = 1420

        setTunnelNetworkSettings(settings) { [weak self] error in
            if let error = error {
                completionHandler(error)
                return
            }

            guard let self = self else { return }
            self.isRunning = true

            // Extract TUN socket file descriptor from packetFlow
            if let fd = self.packetFlow.value(forKeyPath: "socket.fileDescriptor") as? Int32 {
                DispatchQueue.global(qos: .userInteractive).async {
                    let configYaml = """
                    tunnel:
                      name: utun
                      mtu: 1420
                    socks5:
                      port: 1080
                      address: 127.0.0.1
                    misc:
                      task-stack-size: 81920
                      connect-timeout: 5000
                      read-write-timeout: 60000
                    """
                    let tempConfigPath = NSTemporaryDirectory() + "hev_tunnel.yml"
                    try? configYaml.write(toFile: tempConfigPath, atomically: true, encoding: .utf8)

                    hev_socks5_tunnel_main(tempConfigPath, fd)
                }
            }

            completionHandler(nil)
        }
    }

    override func stopTunnel(with reason: NEProviderStopReason, completionHandler: @escaping () -> Void) {
        isRunning = false
        hev_socks5_tunnel_quit()
        completionHandler()
    }
}
