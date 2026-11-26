import Foundation
import NetworkExtension
import Combine

public enum ConnectionState: String {
    case disconnected = "Disconnected"
    case connecting = "Connecting..."
    case connected = "Connected"
    case disconnecting = "Disconnecting..."
    case error = "Error"
}

public enum AppleOperatingMode: String, CaseIterable, Identifiable {
    case autoPilot = "auto_pilot"
    case stealthTunnel = "stealth_tunnel"
    case dpiDesync = "dpi_desync"
    case warpShield = "warp_shield"
    case hybridShield = "hybrid_shield"

    public var id: String { rawValue }

    public var title: String {
        switch self {
        case .autoPilot: return "Auto-Pilot"
        case .stealthTunnel: return "Stealth Tunnel (TCP 443)"
        case .dpiDesync: return "ByeDPI Desync"
        case .warpShield: return "WARP Privacy Shield"
        case .hybridShield: return "Hybrid Shield"
        }
    }

    public var subtitle: String {
        switch self {
        case .autoPilot: return "Autonomous 4-stage empirical bypass ladder"
        case .stealthTunnel: return "Encrypted VLESS over TCP 443 for hostile Wi-Fi"
        case .dpiDesync: return "Local TCP/TLS SNI packet fragmentation"
        case .warpShield: return "WireGuard encrypted Anycast relay"
        case .hybridShield: return "Chained SNI desynchronization with encrypted relay"
        }
    }
}

public class TunnelManager: ObservableObject {
    public static let shared = TunnelManager()

    @Published public var state: ConnectionState = .disconnected
    @Published public var selectedMode: AppleOperatingMode = .autoPilot
    @Published public var latencyMs: Double = 42.0
    @Published public var publicIp: String = "Detecting..."
    @Published public var colo: String = "Detecting..."
    @Published public var bytesRx: UInt64 = 0
    @Published public var bytesTx: UInt64 = 0

    private var providerManager: NETunnelProviderManager?
    private var cancellables = Set<AnyCancellable>()

    private init() {
        loadTunnelPreferences()
        observeSystemVpnStatus()
    }

    public func loadTunnelPreferences() {
        NETunnelProviderManager.loadAllFromPreferences { [weak self] managers, error in
            guard let self = self else { return }
            if let existing = managers?.first {
                self.providerManager = existing
            } else {
                let manager = NETunnelProviderManager()
                let proto = NETunnelProviderProtocol()
                proto.providerBundleIdentifier = "org.netfreedom.app.PacketTunnel"
                proto.serverAddress = "127.0.0.1"
                manager.protocolConfiguration = proto
                manager.localizedDescription = "NetFreedom Universal Shield"
                manager.isEnabled = true
                manager.saveToPreferences { err in
                    if err == nil {
                        self.providerManager = manager
                    }
                }
            }
        }
    }

    public func toggleConnection() {
        if state == .connected || state == .connecting {
            disconnect()
        } else {
            connect()
        }
    }

    public func connect() {
        state = .connecting
        guard let manager = providerManager else {
            state = .error
            return
        }

        let options: [String: NSObject] = [
            "mode": selectedMode.rawValue as NSString,
            "localPort": 1080 as NSNumber
        ]

        do {
            try manager.connection.startVPNTunnel(options: options)
            state = .connected
        } catch {
            state = .error
        }
    }

    public func disconnect() {
        state = .disconnecting
        providerManager?.connection.stopVPNTunnel()
        state = .disconnected
    }

    private func observeSystemVpnStatus() {
        NotificationCenter.default.addObserver(
            forName: .NEVPNStatusDidChange,
            object: nil,
            queue: .main
        ) { [weak self] _ in
            guard let self = self, let conn = self.providerManager?.connection else { return }
            switch conn.status {
            case .connected:
                self.state = .connected
            case .connecting:
                self.state = .connecting
            case .disconnected, .invalid:
                self.state = .disconnected
            case .disconnecting:
                self.state = .disconnecting
            case .reasserting:
                self.state = .connecting
            @unknown default:
                break
            }
        }
    }
}
