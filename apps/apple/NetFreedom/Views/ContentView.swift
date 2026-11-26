import SwiftUI

struct ContentView: View {
    @EnvironmentObject var tunnelManager: TunnelManager
    @State private var selectedTab = 0

    var body: some View {
        TabView(selection: $selectedTab) {
            DashboardView()
                .tabItem {
                    Label("Dashboard", systemImage: "bolt.shield.fill")
                }
                .tag(0)

            ProfilesView()
                .tabItem {
                    Label("Profiles", systemImage: "slider.horizontal.3")
                }
                .tag(1)

            DiagnosticsView()
                .tabItem {
                    Label("Diagnostics", systemImage: "waveform.path.ecg")
                }
                .tag(2)

            SettingsView()
                .tabItem {
                    Label("Settings", systemImage: "gearshape.fill")
                }
                .tag(3)
        }
        .background(Color(red: 0.06, green: 0.09, blue: 0.16))
    }
}
