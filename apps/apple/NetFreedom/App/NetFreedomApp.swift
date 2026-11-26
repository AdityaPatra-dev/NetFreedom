import SwiftUI
import NetworkExtension

@main
struct NetFreedomApp: App {
    @StateObject private var tunnelManager = TunnelManager.shared

    var body: some Scene {
        WindowGroup {
            ContentView()
                .environmentObject(tunnelManager)
                .preferredColorScheme(.dark)
        }
        #if os(macOS)
        .windowStyle(.hiddenTitleBar)
        .defaultSize(width: 960, height: 640)
        #endif
    }
}
