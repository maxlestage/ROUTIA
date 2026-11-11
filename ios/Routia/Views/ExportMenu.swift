import RoutiaCore
import SwiftUI

/// Exporter le bilan : en texte, ou en image dessinée par SwiftUI.
struct ExportMenu: View {
    let habits: [HabitState]
    let colors: [Color]
    let today: Day

    var body: some View {
        Menu {
            ShareLink(item: exportText(habits, today: today)) {
                Label("Partager en texte", systemImage: "doc.text")
            }
            if let image = renderImage() {
                ShareLink(item: image, preview: SharePreview("Bilan Routia", image: image)) {
                    Label("Partager en image", systemImage: "photo")
                }
            }
        } label: {
            Label("Exporter", systemImage: "square.and.arrow.up")
        }
        .disabled(habits.isEmpty)
    }

    @MainActor
    private func renderImage() -> Image? {
        let renderer = ImageRenderer(content: ExportCard(habits: habits, colors: colors, today: today))
        renderer.scale = 3
        guard let image = renderer.uiImage else { return nil }
        return Image(uiImage: image)
    }
}

/// L'image du bilan : une carte par habitude, avec ses 35 derniers jours.
struct ExportCard: View {
    let habits: [HabitState]
    let colors: [Color]
    let today: Day

    var body: some View {
        let date = today.ymd
        VStack(alignment: .leading, spacing: 16) {
            Text("Routia").font(.system(size: 34, weight: .heavy))
            Text("Bilan du \(date.day) \(Day.monthSymbols[date.month - 1]) \(date.year)").foregroundStyle(.secondary)
            ForEach(Array(habits.enumerated()), id: \.offset) { i, h in
                let s = h.streak(today: today)
                let color = colors.indices.contains(i) ? colors[i] : .purple
                VStack(alignment: .leading, spacing: 10) {
                    HStack {
                        Text(h.name).font(.headline)
                        Spacer()
                        Label("\(s) \(h.unit.label(s))", systemImage: "flame.fill")
                            .foregroundStyle(color)
                            .font(.headline)
                    }
                    HStack(spacing: 3) {
                        ForEach(h.history(today: today, days: 35), id: \.day) { item in
                            DayCellView(cell: item.cell, isToday: false, color: color)
                        }
                    }
                }
                .padding(14)
                .background(.white.opacity(0.06), in: .rect(cornerRadius: 18))
            }
        }
        .padding(24)
        .frame(width: 420)
        .background(Palette.background)
        .foregroundStyle(.white)
        .environment(\.colorScheme, .dark)
    }
}
