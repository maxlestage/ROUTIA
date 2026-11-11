import RoutiaCore
import SwiftUI

/// L'anneau du jour et la meilleure série.
struct SummaryCard: View {
    let done: Int
    let total: Int
    let best: (name: String, value: Int, unit: StreakUnit)?

    private var progress: Double { total == 0 ? 0 : Double(done) / Double(total) }

    var body: some View {
        HStack(spacing: 18) {
            ZStack {
                Circle().stroke(.white.opacity(0.08), lineWidth: 10)
                Circle()
                    .trim(from: 0, to: progress)
                    .stroke(Palette.gradient, style: StrokeStyle(lineWidth: 10, lineCap: .round))
                    .rotationEffect(.degrees(-90))
                    .animation(.spring(duration: 0.9, bounce: 0.3), value: progress)
                Text("\(done)/\(total)")
                    .font(.headline.monospacedDigit())
                    .contentTransition(.numericText())
            }
            .frame(width: 76, height: 76)

            VStack(alignment: .leading, spacing: 4) {
                Text("AUJOURD'HUI")
                    .font(.caption.weight(.semibold))
                    .foregroundStyle(.secondary)
                    .tracking(1.4)
                Text(title)
                    .font(.title3.bold())
                    .contentTransition(.interpolate)
                if let best {
                    Label("Meilleure série : \(best.value) \(best.unit.label(best.value)) · \(best.name)", systemImage: "flame.fill")
                        .font(.footnote)
                        .foregroundStyle(.secondary)
                        .symbolEffect(.bounce, value: best.value)
                } else {
                    Text("Cochez un jour pour lancer une série.")
                        .font(.footnote)
                        .foregroundStyle(.secondary)
                }
            }
            Spacer(minLength: 0)
        }
        .padding(18)
        .glassEffect(.regular, in: .rect(cornerRadius: 24))
        .animation(.snappy, value: done)
    }

    private var title: String {
        switch (total, done) {
        case (0, _): "Votre première habitude vous attend"
        case let (t, d) where d == t: "Tout est fait. Bravo ! 🎉"
        case let (t, d): "Encore \(t - d) à faire"
        }
    }
}
