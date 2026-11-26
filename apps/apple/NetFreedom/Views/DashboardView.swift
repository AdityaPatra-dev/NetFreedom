import SwiftUI

struct DashboardView: View {
    @EnvironmentObject var tm: TunnelManager

    var isConnected: Bool {
        tm.state == .connected
    }

    var body: some View {
        VStack(spacing: 24) {
            // Header
            HStack {
                VStack(alignment: .leading, spacing: 4) {
                    Text("NetFreedom Universal")
                        .font(.title2)
                        .fontWeight(.bold)
                        .foregroundColor(.white)
                    Text(tm.selectedMode.title)
                        .font(.subheadline)
                        .foregroundColor(.cyan)
                }
                Spacer()
                Circle()
                    .fill(isConnected ? Color.green : Color.gray)
                    .frame(width: 12, height: 12)
                    .overlay(
                        Circle()
                            .stroke(isConnected ? Color.green.opacity(0.5) : Color.clear, lineWidth: 6)
                    )
            }
            .padding(.horizontal)

            Spacer()

            // Power Connect Button
            Button(action: {
                tm.toggleConnection()
            }) {
                ZStack {
                    Circle()
                        .fill(isConnected ? Color.green.opacity(0.2) : Color.cyan.opacity(0.15))
                        .frame(width: 180, height: 180)
                    Circle()
                        .fill(isConnected ? Color.green : Color.cyan)
                        .frame(width: 140, height: 140)
                    Image(systemName: "power")
                        .font(.system(size: 56, weight: .bold))
                        .foregroundColor(.black)
                }
            }
            .buttonStyle(.plain)

            Text(tm.state.rawValue)
                .font(.headline)
                .foregroundColor(isConnected ? .green : .gray)

            Spacer()

            // Telemetry Grid
            HStack(spacing: 16) {
                TelemetryCard(title: "Handshake RTT", value: "\(String(format: "%.1f", tm.latencyMs)) ms", icon: "timer")
                TelemetryCard(title: "Public IP", value: tm.publicIp, icon: "network")
                TelemetryCard(title: "Edge Gateway", value: tm.colo, icon: "globe.americas.fill")
            }
            .padding(.horizontal)
            .padding(.bottom, 20)
        }
        .padding(.top)
        .background(Color(red: 0.06, green: 0.09, blue: 0.16).ignoresSafeArea())
    }
}

struct TelemetryCard: View {
    let title: String
    let value: String
    let icon: String

    var body: some View {
        VStack(alignment: .leading, spacing: 8) {
            HStack {
                Image(systemName: icon)
                    .foregroundColor(.cyan)
                Text(title)
                    .font(.caption)
                    .foregroundColor(.gray)
            }
            Text(value)
                .font(.subheadline)
                .fontWeight(.bold)
                .foregroundColor(.white)
                .lineLimit(1)
        }
        .frame(maxWidth: .infinity, alignment: .leading)
        .padding(12)
        .background(Color(red: 0.11, green: 0.16, blue: 0.23))
        .cornerRadius(12)
    }
}
