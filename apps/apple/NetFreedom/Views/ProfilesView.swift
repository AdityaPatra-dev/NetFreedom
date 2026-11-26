import SwiftUI

struct ProfilesView: View {
    @EnvironmentObject var tm: TunnelManager

    var body: some View {
        ScrollView {
            VStack(spacing: 16) {
                Text("Select Network Profile")
                    .font(.title3)
                    .fontWeight(.bold)
                    .foregroundColor(.white)
                    .frame(maxWidth: .infinity, alignment: .leading)
                    .padding(.horizontal)

                ForEach(AppleOperatingMode.allCases) { mode in
                    let isSelected = tm.selectedMode == mode
                    VStack(alignment: .leading, spacing: 8) {
                        HStack {
                            Text(mode.title)
                                .font(.headline)
                                .foregroundColor(isSelected ? .cyan : .white)
                            Spacer()
                            if isSelected {
                                Text("ACTIVE")
                                    .font(.caption2)
                                    .fontWeight(.bold)
                                    .padding(.horizontal, 8)
                                    .padding(.vertical, 4)
                                    .background(Color.green.opacity(0.2))
                                    .foregroundColor(.green)
                                    .cornerRadius(6)
                            }
                        }
                        Text(mode.subtitle)
                            .font(.caption)
                            .foregroundColor(.gray)

                        if !isSelected {
                            Button(action: {
                                tm.selectedMode = mode
                                if tm.state == .connected {
                                    tm.disconnect()
                                    DispatchQueue.main.asyncAfter(deadline: .now() + 0.5) {
                                        tm.connect()
                                    }
                                }
                            }) {
                                Text("Activate Profile")
                                    .font(.caption)
                                    .fontWeight(.semibold)
                                    .foregroundColor(.black)
                                    .padding(.horizontal, 12)
                                    .padding(.vertical, 6)
                                    .background(Color.cyan)
                                    .cornerRadius(8)
                            }
                            .buttonStyle(.plain)
                            .padding(.top, 4)
                        }
                    }
                    .padding()
                    .frame(maxWidth: .infinity, alignment: .leading)
                    .background(Color(red: 0.11, green: 0.16, blue: 0.23))
                    .cornerRadius(12)
                    .overlay(
                        RoundedRectangle(cornerRadius: 12)
                            .stroke(isSelected ? Color.cyan : Color.clear, lineWidth: 1.5)
                    )
                    .padding(.horizontal)
                }
            }
            .padding(.vertical)
        }
        .background(Color(red: 0.06, green: 0.09, blue: 0.16).ignoresSafeArea())
    }
}
