import RoutiaCore
import SwiftUI

/// Une habitude : sa série, le bouton du jour et cinq semaines d'historique.
struct HabitCard: View {
    let habit: HabitRecord
    let today: Day
    let onDelete: () -> Void

    @State private var confirmDelete = false
    @State private var celebrate = 0

    var body: some View {
        let state = habit.state
        let streak = state.streak(today: today)
        let done = state.isDone(today)

        VStack(alignment: .leading, spacing: 16) {
            HStack(alignment: .top) {
                VStack(alignment: .leading, spacing: 4) {
                    Text(state.name).font(.title3.bold())
                    Text("\(state.frequencyLabel) · record \(state.bestStreak(today: today)) · \(state.rate(today: today)) % sur 30 j")
                        .font(.footnote)
                        .foregroundStyle(.secondary)
                }
                Spacer(minLength: 8)
                Flame(streak: streak, unit: state.unit, color: habit.color)
            }

            Button {
                withAnimation(.spring(duration: 0.5, bounce: 0.45)) { habit.toggle(today) }
                if !done { celebrate += 1 }
            } label: {
                HStack(spacing: 12) {
                    Image(systemName: done ? "checkmark.square.fill" : "square")
                        .font(.title2)
                        .foregroundStyle(habit.color)
                        .contentTransition(.symbolEffect(.replace))
                    Text(done ? "Fait aujourd'hui" : "Marquer comme fait")
                        .font(.headline)
                }
                .frame(maxWidth: .infinity, minHeight: 52)
            }
            .buttonStyle(.glass)
            .tint(done ? habit.color : nil)
            .sensoryFeedback(.success, trigger: celebrate)
            .accessibilityAddTraits(done ? .isSelected : [])

            HistoryGrid(state: state, today: today, color: habit.color) { day in
                withAnimation(.snappy) { habit.toggle(day) }
            }

            HStack {
                Text("\(state.missed(today: today)) jour(s) manqué(s) sur 30 jours")
                    .font(.caption)
                    .foregroundStyle(.tertiary)
                Spacer()
                Button("Supprimer", role: .destructive) { confirmDelete = true }
                    .font(.footnote)
            }
        }
        .padding(18)
        .overlay(alignment: .leading) {
            Capsule()
                .fill(habit.color)
                .frame(width: 4)
                .padding(.vertical, 18)
                .shadow(color: habit.color, radius: 8)
        }
        .glassEffect(.regular.tint(habit.color.opacity(done ? 0.18 : 0.06)), in: .rect(cornerRadius: 24))
        // Un petit rebond à plat quand la journée est cochée.
        .keyframeAnimator(initialValue: 1.0, trigger: celebrate) { content, scale in
            content.scaleEffect(scale)
        } keyframes: { _ in
            KeyframeTrack {
                SpringKeyframe(1.04, duration: 0.16)
                SpringKeyframe(0.98, duration: 0.18)
                SpringKeyframe(1.0, duration: 0.3)
            }
        }
        .confirmationDialog("Supprimer « \(state.name) » ?", isPresented: $confirmDelete, titleVisibility: .visible) {
            Button("Supprimer", role: .destructive, action: onDelete)
        }
    }
}

/// La flamme de la série : elle danse tant que la série tient.
struct Flame: View {
    let streak: Int
    let unit: StreakUnit
    let color: Color

    var body: some View {
        VStack(spacing: 0) {
            Image(systemName: "flame.fill")
                .font(.title3)
                .foregroundStyle(streak > 0 ? AnyShapeStyle(Color.orange.gradient) : AnyShapeStyle(Color.gray))
                .symbolEffect(.breathe, isActive: streak > 0)
            Text("\(streak)")
                .font(.title2.weight(.heavy).monospacedDigit())
                .contentTransition(.numericText(value: Double(streak)))
            Text(unit.label(streak))
                .font(.caption2)
                .foregroundStyle(.secondary)
        }
        .frame(minWidth: 64)
        .padding(.vertical, 8)
        .glassEffect(.regular.tint(streak > 0 ? .orange.opacity(0.15) : .clear), in: .rect(cornerRadius: 16))
        .accessibilityElement(children: .combine)
        .accessibilityLabel("Série : \(streak) \(unit.label(streak))")
    }
}
