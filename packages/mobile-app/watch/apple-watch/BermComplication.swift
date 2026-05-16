// BermComplication.swift
// Apple Watch widget (WidgetKit) for BERM Alert.
//
// Integration: add a Widget Extension target to the iOS app produced by
// `expo prebuild`, then include this file. The shared App Group
// "group.sh.berm.alert" carries the latest cover snapshot written by the
// React Native layer (see watch/README.md). Builds with the main iOS app via
// `eas build --platform ios`.

import WidgetKit
import SwiftUI

struct CoverSnapshot: Codable {
    let riskScore: Int
    let band: String
    let activeCovers: Int
    let unreadAlerts: Int
    let updatedAt: Date
}

private let appGroup = "group.sh.berm.alert"
private let snapshotKey = "berm.coverSnapshot"

private let breakwaterGrey = Color(red: 0.165, green: 0.165, blue: 0.165)
private let glowCyan = Color(red: 0.357, green: 0.753, blue: 0.922)
private let safetyAmber = Color(red: 1.0, green: 0.851, blue: 0.239)

func loadSnapshot() -> CoverSnapshot {
    guard
        let defaults = UserDefaults(suiteName: appGroup),
        let data = defaults.data(forKey: snapshotKey),
        let snapshot = try? JSONDecoder().decode(CoverSnapshot.self, from: data)
    else {
        return CoverSnapshot(riskScore: 0, band: "CALM", activeCovers: 0, unreadAlerts: 0, updatedAt: Date())
    }
    return snapshot
}

func riskColor(_ score: Int) -> Color {
    switch score {
    case 75...: return Color(red: 0.898, green: 0.325, blue: 0.294)
    case 50..<75: return Color(red: 1.0, green: 0.702, blue: 0.278)
    case 25..<50: return safetyAmber
    default: return glowCyan
    }
}

struct CoverEntry: TimelineEntry {
    let date: Date
    let snapshot: CoverSnapshot
}

struct CoverProvider: TimelineProvider {
    func placeholder(in context: Context) -> CoverEntry {
        CoverEntry(date: Date(), snapshot: loadSnapshot())
    }

    func getSnapshot(in context: Context, completion: @escaping (CoverEntry) -> Void) {
        completion(CoverEntry(date: Date(), snapshot: loadSnapshot()))
    }

    func getTimeline(in context: Context, completion: @escaping (Timeline<CoverEntry>) -> Void) {
        let entry = CoverEntry(date: Date(), snapshot: loadSnapshot())
        let next = Calendar.current.date(byAdding: .minute, value: 15, to: Date()) ?? Date()
        completion(Timeline(entries: [entry], policy: .after(next)))
    }
}

struct BermComplicationView: View {
    @Environment(\.widgetFamily) var family
    let entry: CoverEntry

    var body: some View {
        switch family {
        case .accessoryCircular:
            ZStack {
                Gauge(value: Double(entry.snapshot.riskScore), in: 0...100) {
                    Text("BERM")
                } currentValueLabel: {
                    Text("\(entry.snapshot.riskScore)")
                        .foregroundColor(riskColor(entry.snapshot.riskScore))
                }
                .gaugeStyle(.accessoryCircular)
                .tint(riskColor(entry.snapshot.riskScore))
            }
        case .accessoryInline:
            Text("BERM \(entry.snapshot.band) \(entry.snapshot.riskScore)")
                .foregroundColor(riskColor(entry.snapshot.riskScore))
        default:
            HStack(spacing: 6) {
                Text("\(entry.snapshot.riskScore)")
                    .font(.system(size: 22, weight: .bold))
                    .foregroundColor(riskColor(entry.snapshot.riskScore))
                VStack(alignment: .leading) {
                    Text(entry.snapshot.band).font(.caption).foregroundColor(glowCyan)
                    Text("\(entry.snapshot.activeCovers) covers").font(.caption2).foregroundColor(.gray)
                }
            }
        }
    }
}

@main
struct BermComplication: Widget {
    let kind = "BermComplication"

    var body: some WidgetConfiguration {
        StaticConfiguration(kind: kind, provider: CoverProvider()) { entry in
            BermComplicationView(entry: entry)
                .containerBackground(breakwaterGrey, for: .widget)
        }
        .configurationDisplayName("BERM Alert")
        .description("Cover risk and active covers at a glance.")
        .supportedFamilies([.accessoryCircular, .accessoryInline, .accessoryRectangular])
    }
}
