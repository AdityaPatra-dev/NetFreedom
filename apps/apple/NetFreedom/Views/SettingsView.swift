import SwiftUI

struct SettingsView: View {
    @AppStorage("netfreedom_killswitch") private var killSwitch = false
    @AppStorage("netfreedom_dns_provider") private var dnsProvider = "1.1.1.1"
    @AppStorage("netfreedom_mtu") private var mtu = 1420
    @AppStorage("netfreedom_worker_url") private var workerUrl = ""
    @AppStorage("netfreedom_worker_uuid") private var workerUuid = ""

    var body: some View {
        ScrollView {
            VStack(spacing: 20) {
                Text("App & Tunnel Settings")
                    .font(.title3)
                    .fontWeight(.bold)
                    .foregroundColor(.white)
                    .frame(maxWidth: .infinity, alignment: .leading)
                    .padding(.horizontal)

                // Kill Switch Section
                VStack(alignment: .leading, spacing: 12) {
                    Toggle(isOn: $killSwitch) {
                        VStack(alignment: .leading, spacing: 4) {
                            Text("Strict Kill Switch")
                                .font(.headline)
                                .foregroundColor(.white)
                            Text("Block all internet traffic if tunnel unexpectedly disconnects")
                                .font(.caption)
                                .foregroundColor(.gray)
                        }
                    }
                    .toggleStyle(SwitchToggleStyle(tint: .cyan))
                }
                .padding()
                .background(Color(red: 0.11, green: 0.16, blue: 0.23))
                .cornerRadius(12)
                .padding(.horizontal)

                // Stealth Worker Node Section
                VStack(alignment: .leading, spacing: 12) {
                    Text("Custom Stealth Node (Cloudflare Worker)")
                        .font(.headline)
                        .foregroundColor(.white)
                    Text("Enter your deployed VLESS WebSocket worker to bypass strict campus firewalls")
                        .font(.caption)
                        .foregroundColor(.gray)

                    TextField("my-tunnel.workers.dev", text: $workerUrl)
                        .textFieldStyle(.roundedBorder)

                    TextField("VLESS UUID (e.g. d342d110-...)", text: $workerUuid)
                        .textFieldStyle(.roundedBorder)
                }
                .padding()
                .background(Color(red: 0.11, green: 0.16, blue: 0.23))
                .cornerRadius(12)
                .padding(.horizontal)

                // MTU and DNS
                VStack(alignment: .leading, spacing: 12) {
                    Text("Tunnel MTU: \(mtu)")
                        .font(.headline)
                        .foregroundColor(.white)
                    Slider(value: Binding(
                        get: { Double(mtu) },
                        set: { mtu = Int($0) }
                    ), in: 1280...1500, step: 10)
                    .accentColor(.cyan)
                }
                .padding()
                .background(Color(red: 0.11, green: 0.16, blue: 0.23))
                .cornerRadius(12)
                .padding(.horizontal)
            }
            .padding(.vertical)
        }
        .background(Color(red: 0.06, green: 0.09, blue: 0.16).ignoresSafeArea())
    }
}
