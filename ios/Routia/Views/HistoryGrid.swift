import RoutiaCore
import SwiftUI

/// Cinq semaines, du lundi au dimanche ; toucher un jour passé le corrige.
struct HistoryGrid: View {
    let state: HabitState
    let today: Day
    let color: Color
    let toggle: (Day) -> Void

    private let columns = Array(repeating: GridItem(.flexible(), spacing: 5), count: 7)

    var body: some View {
        let first = today.weekStart.adding(-28)
        VStack(spacing: 6) {
            LazyVGrid(columns: columns, spacing: 5) {
                ForEach(Array(Day.weekdaySymbols.enumerated()), id: \.offset) { _, symbol in
                    Text(symbol).font(.caption2).foregroundStyle(.tertiary)
                }
                ForEach(0..<35, id: \.self) { i in
                    let day = first.adding(i)
                    DayCellView(cell: state.cell(day, today: today), isToday: day == today, color: color)
                        .onTapGesture { if day <= today { toggle(day) } }
                        .accessibilityLabel(label(day))
                        .accessibilityAddTraits(.isButton)
                }
            }
        }
    }

    private func label(_ day: Day) -> String {
        let (y, m, d) = day.ymd
        let status = switch state.cell(day, today: today) {
        case .done: "fait"
        case .missed: "manqué"
        case .pending: "à faire"
        case .rest: "repos"
        case .empty: ""
        }
        return "\(d) \(Day.monthSymbols[m - 1]) \(y) \(status)"
    }
}

struct DayCellView: View {
    let cell: DayCell
    let isToday: Bool
    let color: Color

    var body: some View {
        RoundedRectangle(cornerRadius: 7, style: .continuous)
            .fill(fill)
            .aspectRatio(1, contentMode: .fit)
            .overlay {
                if cell == .missed {
                    RoundedRectangle(cornerRadius: 7, style: .continuous)
                        .strokeBorder(Color(red: 0.98, green: 0.44, blue: 0.52).opacity(0.6), style: StrokeStyle(lineWidth: 1, dash: [3]))
                }
                if isToday {
                    RoundedRectangle(cornerRadius: 7, style: .continuous)
                        .strokeBorder(.white, lineWidth: 2)
                }
            }
            .shadow(color: cell == .done ? color.opacity(0.6) : .clear, radius: 4, y: 2)
            .scaleEffect(cell == .done ? 1 : 0.92)
            .animation(.spring(duration: 0.35, bounce: 0.5), value: cell)
    }

    private var fill: Color {
        switch cell {
        case .done: color
        case .missed: Color(red: 0.98, green: 0.44, blue: 0.52).opacity(0.18)
        case .pending: .white.opacity(0.12)
        case .rest: .white.opacity(0.1)
        case .empty: .white.opacity(0.04)
        }
    }
}
