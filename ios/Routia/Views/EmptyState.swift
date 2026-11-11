import SwiftUI

/// Aucune habitude : des anneaux qui se remplissent, et un bouton pour commencer.
struct EmptyState: View {
    let start: () -> Void

    var body: some View {
        VStack(spacing: 20) {
            Rings()
                .frame(height: 220)
            Text("Une habitude à la fois.")
                .font(.title2.bold())
            Text("Créez la première : elle restera sur cet iPhone, sans compte.")
                .multilineTextAlignment(.center)
                .foregroundStyle(.secondary)
            Button("Créer une habitude", systemImage: "plus", action: start)
                .buttonStyle(.glassProminent)
                .controlSize(.large)
        }
        .padding(.vertical, 24)
    }
}

/// Trois anneaux qui se remplissent à plat, comme sur le site, autour d'un noyau qui bat.
struct Rings: View {
    private let sizes: [CGFloat] = [180, 136, 92]
    private let colors: [Int] = [0, 1, 4]

    var body: some View {
        TimelineView(.animation) { timeline in
            let t = timeline.date.timeIntervalSinceReferenceDate
            ZStack {
                ForEach(0..<3, id: \.self) { i in
                    let cycle = (t * 0.22 + Double(i) * 0.2).truncatingRemainder(dividingBy: 1)
                    let fill = cycle < 0.7 ? 1 - pow(1 - cycle / 0.7, 3) : 1
                    Circle()
                        .stroke(.white.opacity(0.08), lineWidth: 14)
                        .frame(width: sizes[i], height: sizes[i])
                    Circle()
                        .trim(from: 0, to: fill)
                        .stroke(
                            Palette.color(colors[i]).gradient,
                            style: StrokeStyle(lineWidth: 14, lineCap: .round)
                        )
                        .frame(width: sizes[i], height: sizes[i])
                        .rotationEffect(.degrees(-90 + t * Double(20 + i * 15)))
                        .shadow(color: Palette.color(colors[i]), radius: 8)
                }
                Circle()
                    .fill(Palette.color(3).gradient)
                    .frame(width: 34, height: 34)
                    .scaleEffect(1 + sin(t * 2.4) * 0.08)
                    .shadow(color: Palette.color(3), radius: 16)
            }
        }
        .accessibilityHidden(true)
    }
}
