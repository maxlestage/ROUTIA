import RoutiaCore
import SwiftData
import SwiftUI

/// L'écran principal : l'anneau du jour, les habitudes, l'ajout et l'export.
struct TodayView: View {
    @Environment(\.modelContext) private var context
    @Query(sort: \HabitRecord.createdAt, order: .reverse) private var habits: [HabitRecord]
    @State private var adding = false
    @State private var today = Day.today

    private var doneCount: Int { habits.filter { $0.state.isDone(today) }.count }

    var body: some View {
        NavigationStack {
            ScrollView {
                VStack(spacing: 16) {
                    SummaryCard(done: doneCount, total: habits.count, best: bestStreak)
                    if habits.isEmpty {
                        EmptyState { adding = true }
                    } else {
                        ForEach(Array(habits.enumerated()), id: \.element.id) { index, habit in
                            HabitCard(habit: habit, today: today) {
                                withAnimation(.snappy) { context.delete(habit) }
                            }
                            .transition(.asymmetric(
                                insertion: .scale(scale: 0.9).combined(with: .opacity),
                                removal: .move(edge: .leading).combined(with: .opacity)
                            ))
                            .scrollTransition { content, phase in
                                content
                                    .opacity(phase.isIdentity ? 1 : 0.4)
                                    .scaleEffect(phase.isIdentity ? 1 : 0.94)
                                    .blur(radius: phase.isIdentity ? 0 : 2)
                            }
                            .animation(.spring(duration: 0.5).delay(Double(index) * 0.04), value: habits.count)
                        }
                    }
                }
                .padding(.horizontal, 16)
                .padding(.bottom, 32)
            }
            .scrollBounceBehavior(.basedOnSize)
            .background(Backdrop())
            .navigationTitle("Aujourd'hui")
            .toolbar {
                ToolbarItem(placement: .topBarLeading) {
                    ExportMenu(habits: habits.map(\.state), colors: habits.map(\.color), today: today)
                }
                ToolbarItem(placement: .topBarTrailing) {
                    Button("Nouvelle habitude", systemImage: "plus") { adding = true }
                        .buttonStyle(.glassProminent)
                }
            }
            .sheet(isPresented: $adding) {
                NewHabitSheet(nextColor: habits.count) { name, frequency, color in
                    withAnimation(.bouncy) {
                        context.insert(HabitRecord(name: name, frequency: frequency, colorIndex: color, today: today))
                    }
                }
                .presentationDetents([.medium, .large])
            }
            .onReceive(NotificationCenter.default.publisher(for: .NSCalendarDayChanged)) { _ in
                today = .today
            }
        }
    }

    private var bestStreak: (name: String, value: Int, unit: StreakUnit)? {
        var best: (name: String, value: Int, unit: StreakUnit)?
        for habit in habits {
            let state = habit.state
            let value = state.streak(today: today)
            if value > (best?.value ?? 0) {
                best = (name: state.name, value: value, unit: state.unit)
            }
        }
        return best
    }
}

/// Le fond : un dégradé nuit et deux halos qui dérivent.
struct Backdrop: View {
    var body: some View {
        TimelineView(.animation) { timeline in
            let t = timeline.date.timeIntervalSinceReferenceDate
            ZStack {
                Palette.background
                Circle()
                    .fill(Palette.color(0).opacity(0.45))
                    .frame(width: 420)
                    .blur(radius: 90)
                    .offset(x: -120 + sin(t / 5) * 40, y: -260 + cos(t / 6) * 30)
                Circle()
                    .fill(Palette.color(1).opacity(0.25))
                    .frame(width: 380)
                    .blur(radius: 100)
                    .offset(x: 150 + cos(t / 7) * 40, y: 200 + sin(t / 4) * 40)
            }
            .ignoresSafeArea()
        }
    }
}

#Preview {
    TodayView()
        .modelContainer(for: HabitRecord.self, inMemory: true)
        .preferredColorScheme(.dark)
}
